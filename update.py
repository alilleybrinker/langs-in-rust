import json
import requests
import datetime
import sys
import os
import locale
import dotenv


def readme_header(file):
    """
    Write the header of the `README.md` being generated.

    This includes an explanation of what the repository is for, and the
    criteria for language inclusion, as well as the header of the Markdown
    table being generated.
    """

    header = (
        "# Languages Written in Rust\n"
        "\n"
        "This is a (probably incomplete) list of languages implemented in\n"
        "Rust. It is intended as a source of inspiration and comparison, and as a\n"
        "directory of potentially interesting projects in this vein.\n"
        "\n"
        "## What Can Be Included?\n"
        "\n"
        "1. Is it a language?\n"
        "2. Is it written in Rust?\n"
        "\n"
        "Then it can be included in this list!\n"
        "\n"
        "## List of Languages\n"
        "\n"
        "| Name | ⭐ Stars | ☀️ Status | Description |\n"
        "|:-----|:---------|:-----------|:-----------|\n"
    )

    file.write(header)


def readme_footer(file):
    """
    Write the footer of the `README.md` being generated.

    This includes any additional information which is to be written _after_ the
    table of languages.
    """

    footer = (
        "\n"
        "*: Parcel is a large project of which the JavaScript transformer (written in Rust)\n"
        'is a small part. The "stars" number here reflects the whole project, which is\n'
        "broader than a programming language project.\n"
        "\n"
    )

    file.write(footer)


def key(e):
    """
    Gets the star count for an entry in the language list.

    This is used to sort languages when the table is generated.
    """

    return e["stars"]


def extract_languages():
    """
    Load languages from the `languages.json` file, returning
    `active_langs`, `inactive_langs`, and `lang_names` to be
    used when generating the language table.
    """

    with open("languages.json", "r", encoding="utf-8") as file:
        data = json.load(file)

        active_langs = []
        inactive_langs = []
        lang_names = []

        for lang in data:
            lang_names.append({"name": lang["name"], "url": lang["url"]})

            if lang["active"]:
                active_langs.append(lang)
            else:
                inactive_langs.append(lang)

        active_langs.sort(reverse=True, key=key)
        inactive_langs.sort(reverse=True, key=key)

        return active_langs, inactive_langs, lang_names


def api_request(token):
    """
    Update `languages.json` based on the latest API data for each language.

    Note that this only updates the 'activity' and 'stars' values.
    """

    with open("languages.json", "r+", encoding="utf-8") as file:
        # Read the current state of the file.
        languages = json.load(file)
        new_data = []

        for lang in languages:
            # TODO: Also support requests to other APIs like GitLab to get this info.
            if "github.com" in lang["url"]:
                # Get the URL for the proper API request.
                url = lang["url"].replace("github.com", "api.github.com/repos")

                # Make the API request with the passed-in token.
                response = requests.get(
                    url, headers={f"Authorization": f"token {token}"}
                )
                # Print an error if something fails, but keep going.
                if response.status_code != 200:
                    print(f"API request failed for {url}", file=sys.stderr)
                    continue

                # Get the latest data and update the list.
                data = response.json()
                new_data.append(
                    {
                        "name": lang["name"],
                        "description": lang["description"],
                        "url": lang["url"],
                        "stars": data["stargazers_count"],
                        "active": is_active(data["pushed_at"]),
                    }
                )

        # Update the file.
        file.seek(0)
        json.dump(new_data, file, indent=4)
        file.truncate()


def is_active(raw_updated_at):
    """
    Determine whether a project has been updated recently enough to treat as
    "active" in the table.
    """

    updated_at = datetime.datetime.strptime(raw_updated_at, "%Y-%m-%dT%H:%M:%SZ")
    delta = datetime.timedelta(weeks=24)
    date = updated_at + delta
    now = datetime.datetime.now()
    return date >= now


def write_readme():
    """
    Write the current contents of `languages.json` out to the `README.md` file.
    """

    # Get the current data from the JSON file.
    active, inactive, languages = extract_languages()

    with open("README.md", "w", encoding="utf-8") as out:
        out.seek(0)

        # Write the header.
        readme_header(out)

        # Write the active language entries.
        for i in active:
            out.write(
                f"| [{i['name']}] | {i['stars']:,d} | ☀️ Active | {i['description']} |\n"
            )

        # Write the inactive language entries.
        for i in inactive:
            out.write(
                f"| [{i['name']}] | {i['stars']:,d} | 🌙 Inactive | {i['description']} |\n"
            )

        # Write the footer.
        readme_footer(out)

        # Write the Markdown link entries.
        for i in languages:
            out.write(f"[{i['name']}]: {i['url']}\n")

        out.truncate()


if __name__ == "__main__":
    dotenv.load_dotenv()

    try:
        if len(sys.argv) != 2:
            print("usage: python update.py [api] [readme]")
            exit(1)
        elif sys.argv[1] == "readme":
            write_readme()
            exit(0)
        elif sys.argv[1] == "api":
            api_request(os.getenv("GITHUB_API_TOKEN"))
            write_readme()
            exit(0)
        else:
            print(f"error: unknown flag: {sys.argv[1]}")
            exit(1)
    except Exception as ex:
        print(f"error: {ex}")
        exit(1)
