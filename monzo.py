# %%
import json
import os
import pyperclip
import requests
from collections import namedtuple
from urllib import parse

# %%
access_token = os.getenv("MONZO_ACCESS_TOKEN") or input("Enter access token:")
account_id = os.getenv("MONZO_ACCOUNT_ID") or input("Enter account id:")
destination_url = "https://monzo2discord.azurewebsites.net/api/HttpExample"
with open("data/monzo.json") as f:
    monzo: dict = json.load(f)

# %%
# List webhooks
response = requests.get(
    url="https://api.monzo.com/webhooks",
    headers={"Authorization": f"Bearer {access_token}"},
    params={"account_id": account_id},
)
print(response.content)
assert response.ok

# %%
# Register a webhook
response = requests.post(
    url="https://api.monzo.com/webhooks",
    headers={"Authorization": f"Bearer {access_token}"},
    data={"account_id": account_id, "url": destination_url},
)
print(response.content)
assert response.ok

# %%
# Delete a webhook
webhook_id = input("Webhook id to delete:")
response = requests.delete(
    url=f"https://api.monzo.com/webhooks/{webhook_id}",
    headers={"Authorization": f"Bearer {access_token}"},
)
print(response.content)
assert response.ok

# %%
# Emulate Monzo's POST:
example_body = {
    "type": "transaction.created",
    "data": {
        "account_id": "acc_00008gju41AHyfLUzBUk8A",
        "amount": -350,
        "created": "2015-09-04T14:28:40Z",
        "currency": "GBP",
        "description": "Ozone Coffee Roasters",
        "id": "tx_00008zjky19HyFLAzlUk7t",
        "category": "eating_out",
        "is_load": False,
        "settled": "2015-09-05T14:28:40Z",
        "merchant": {
            "address": {
                "address": "98 Southgate Road",
                "city": "London",
                "country": "GB",
                "latitude": 51.54151,
                "longitude": -0.08482400000002599,
                "postcode": "N1 3JD",
                "region": "Greater London",
            },
            "created": "2015-08-22T12:20:18Z",
            "group_id": "grp_00008zIcpbBOaAr7TTP3sv",
            "id": "merch_00008zIcpbAKe8shBxXUtl",
            "logo": "https://pbs.twimg.com/profile_images/527043602623389696/68_SgUWJ.jpeg",
            "emoji": "🍞",
            "name": "The De Beauvoir Deli Co.",
            "category": "eating_out",
        },
    },
}
test_post = input("Post to:") or destination_url
response = requests.post(url=test_post, json=example_body)
print(response.content)
assert response.ok

# %%
# Enter the flow Monzo side
query = parse.urlencode(
    {
        "client_id": monzo["client_id"],
        "redirect_uri": "https://aatifsyed.uk",
        "response_type": "code",
        "state": 1234,
    }
)

Url = namedtuple("Url", ["scheme", "netloc", "path", "params", "query", "fragment"])

pre_auth_url = Url(
    scheme="https",
    netloc="auth.monzo.com",
    path="",
    params="",
    query=query,
    fragment="",
)

pyperclip.copy(parse.urlunparse(pre_auth_url))

# %%
# Exchange the authorization code for an access token
authorized_url = input("Paste authorized URL:")
authorization_code = parse.parse_qs(parse.urlparse(authorized_url).query)["code"]
response = requests.post(
    url="https://api.monzo.com/oauth2/token",
    data={
        "grant_type": "authorization_code",
        "client_id": monzo["client_id"],
        "client_secret": monzo["client_secret"],
        "redirect_uri": "https://aatifsyed.uk",
        "code": authorization_code,
    },
)
print(response.content)
assert response.ok

access_token = response.json()["access_token"]
refresh_token = response.json()["refresh_token"]

# %%
# Write to file
with open("data/state.json", "w") as f:
    json.dump({"access_token": access_token, "refresh_token": refresh_token}, f)
# %%
