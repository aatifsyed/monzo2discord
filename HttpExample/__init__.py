import json
import logging
import requests

import azure.functions as func


def main(
    request: func.HttpRequest,
    discordjson: func.InputStream,
    monzojson: func.InputStream,
) -> func.HttpResponse:
    logging.info("Python HTTP trigger function processed a request.")

    logging.info(monzojson := json.load(monzojson))
    logging.info(discordjson := json.load(discordjson))

    requests.post(url=discordjson["url"], json={"content": "foo"})

    return func.HttpResponse(
        "This HTTP triggered function executed successfully. Pass a name in the query string or in the request body for a personalized response.",
        status_code=200,
    )
