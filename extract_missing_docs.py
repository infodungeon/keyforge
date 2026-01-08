import json
import sys

def extract_missing_docs(filename):
    missing_docs = []
    with open(filename, 'r') as f:
        for line in f:
            try:
                data = json.loads(line)
                if data.get('reason') == 'compiler-message':
                    msg = data.get('message', {})
                    if msg.get('code', {}).get('code') == 'missing_docs':
                        spans = msg.get('spans', [])
                        if spans:
                            span = spans[0]
                            missing_docs.append({
                                'file': span['file_name'],
                                'line': span['line_start'],
                                'message': msg['message'],
                                'text': span['text'][0]['text'] if span['text'] else ''
                            })
            except json.JSONDecodeError:
                continue
    return missing_docs

if __name__ == '__main__':
    docs = extract_missing_docs('check_output.json')
    for doc in docs:
        print(f"{doc['file']}:{doc['line']} - {doc['message']} - {doc['text']}")
