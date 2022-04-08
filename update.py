import json
import requests
import datetime
import sys
import os
import locale
from dotenv import load_dotenv

locale.setlocale(locale.LC_ALL, '')
load_dotenv()


def readme_top(file):
    file.write('# Languages Written in Rust\n\n')
    file.write('This is a (probably incomplete) list of languages implemented in\n')
    file.write('Rust. It is intended as a source of inspiration and comparison, and as a')
    file.write('directory of potentially interesting projects in this vein.\n\n')
    file.write('## What Can Be Included?\n\n')
    file.write('1. Is it a language?\n')
    file.write('2. Is it written in Rust?\n\n')
    file.write('Then it can be included in this list!\n\n')
    file.write('## List of Languages\n\n')
    file.write('| Name | ⭐ Stars | ☀️ Status | Description |\n')
    file.write('|:-----|:---------|:-----------|:-----------|\n')


def readme_bottom(file):
    file.write("\n*: Parcel is a large project of which the JavaScript transformer (written in Rust)\n")
    file.write("is a small part. The \"stars\" number here reflects the whole project, which is\n")
    file.write("broader than a programming language project.\n\n")


def key(e):
    return e['stars']


def extract_languages():
    with open("languages.json", 'r', encoding='utf-8') as file:
        data = json.load(file)
        active_langs = []
        inactive_langs = []
        langs_names = []
        for i in data:
            langs_names.append({
                'name': i["name"],
                'url': i["url"]
            })
            if i["active"]:
                active_langs.append(i)
                continue
            inactive_langs.append(i)
        active_langs.sort(reverse=True, key=key)
        inactive_langs.sort(reverse=True, key=key)
        return active_langs, inactive_langs, langs_names


def api_request(token):
    with open("languages.json", 'r+', encoding='utf-8') as file:
        languages = json.load(file)
        count = 0
        for i in languages:
            count += 1
            if "github.com" in i["url"]:
                url = i["url"].replace("github.com", "api.github.com/repos")
                response = requests.get(url,
                                        headers={f'Authorization': f'token {token}'})
                if response.status_code == 200:
                    data = response.json()
                    is_active = True
                    date = datetime.datetime.strptime(data['updated_at'], "%Y-%m-%dT%H:%M:%SZ") + datetime.timedelta(
                        weeks=8 + 8 + 8)
                    now = datetime.datetime.now()
                    if date < now:
                        is_active = False
                    elif date > now:
                        is_active = True
                    else:
                        is_active = True
                    if is_active:
                        lang_obj = {
                            'name': i["name"],
                            'description': data["description"],
                            'url': i["url"],
                            'stars': data['stargazers_count'],
                            'active': is_active
                        }
                        languages[count - 1] = lang_obj
                    if not is_active:
                        lang_obj = {
                            "name": i["name"],
                            "description": data["description"],
                            "url": i["url"],
                            "stars": data['stargazers_count'],
                            "active": is_active
                        }
                        languages[count - 1] = lang_obj
        file.seek(0)
        json.dump(languages, file, indent=4)
        file.truncate()


def write_readme():
    active, inactive, languages = extract_languages()
    with open("README.md", 'w', encoding='utf-8') as out:
        out.seek(0)
        readme_top(out)
        for i in active:
            out.write(f"| [{i['name']}] | {i['stars']:n} | ☀️ Active | {i['description']} |\n")
        for i in inactive:
            out.write(f"| [{i['name']}] | {i['stars']:n} | 🌙 Inactive | {i['description']} |\n")
        readme_bottom(out)
        for i in languages:
            out.write(f"[{i['name']}]: {i['url']}\n")
        out.truncate()


if __name__ == '__main__':
    try:
        if len(sys.argv) < 2:
            write_readme()
            print("Finished.")
            exit(0)
        if sys.argv[1] == 'api':
            api_request(os.getenv('TOKEN'))
            write_readme()
            print("Finished.")
            exit(0)
        else:
            print("Usage: python update.py api")
            print(f"Error: Unknown flag {sys.argv[1]}")
            exit(1)
    except Exception as ex:
        print(ex)
