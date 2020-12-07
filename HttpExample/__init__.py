import json
import logging

import azure.functions as func


def main(
    req: func.HttpRequest, discordjson: func.InputStream, monzojson: func.InputStream
) -> func.HttpResponse:
    logging.info("Python HTTP trigger function processed a request.")

    logging.info(json.load(monzojson))
    logging.info(json.load(discordjson))

    return func.HttpResponse(
        "This HTTP triggered function executed successfully. Pass a name in the query string or in the request body for a personalized response.",
        status_code=200,
    )
