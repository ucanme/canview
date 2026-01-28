import json
import sys

with open('check_errors.json', 'r', encoding='utf-16') as f:
    for line in f:
        try:
            data = json.loads(line)
            if data.get('reason') == 'compiler-message':
                msg = data.get('message')
                if msg.get('level') == 'error':
                    print(f"Error: {msg.get('message')}")
                    print(f"Code: {msg.get('code', {}).get('code')}")
                    for span in msg.get('spans', []):
                        print(f"File: {span.get('file_name')}:{span.get('line_start')}:{span.get('column_start')}")
                        print(f"Text: {span.get('text', [])}")
        except:
            pass
