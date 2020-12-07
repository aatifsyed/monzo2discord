# %%
import os
from types import resolve_bases
import requests

# %%
access_token = os.getenv("MONZO_ACCESS_TOKEN") or input("Enter access token:")
account_id = os.getenv("MONZO_ACCOUNT_ID") or input("Enter account id:")
destination_url = "https://monzo2discord.azurewebsites.net/api/HttpExample"

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
