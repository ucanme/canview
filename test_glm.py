import json

import requests


def verify_glm_api_key(api_key, model="glm-4-flash"):
    """
    验证GLM API Key是否有效

    Parameters:
    api_key: 你的API Key
    model: 使用的模型，默认使用轻量级模型节省费用
    """
    url = "https://open.bigmodel.cn/api/paas/v4/chat/completions"

    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}

    payload = {
        "model": model,
        "messages": [{"role": "user", "content": "请回复'API Key验证成功'"}],
        "max_tokens": 50,
    }

    try:
        response = requests.post(url, headers=headers, json=payload)

        if response.status_code == 200:
            data = response.json()
            print("---data---", data)
            print("✅ API Key 验证成功！")
            print(f"模型: {data.get('model')}")
            print(f"回复: {data['choices'][0]['message']['content']}")
            return True
        elif response.status_code == 401:
            print("❌ API Key 无效或已过期")
            return False
        elif response.status_code == 429:
            print("⚠️  请求过于频繁，请稍后再试")
            return False
        else:
            print(f"❌ 验证失败，状态码: {response.status_code}")
            print(f"错误信息: {response.text}")
            return False

    except Exception as e:
        print(f"❌ 请求失败: {str(e)}")
        return False


# 使用示例
api_key = "8c95a9bf214f44d99afa1721084568e5.EjfTpkcCur9wPfMf"
verify_glm_api_key(api_key)
