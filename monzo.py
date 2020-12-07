# %%
import os
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
