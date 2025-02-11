import uuid
from confidence_shared import *
import asyncio
import time


async def get_flag():
    api_config = ApiConfig(api_key="xa0fQ4WKSvuxdjPtesupleiSbZeik6Gf", region=Region.EU)
    root = Confidence(api_config)
    random_uuid = uuid.uuid4()
    uuid_string = str(random_uuid)
    confidence = root.with_context({"targeting_key": ConfidenceValue.STRING(uuid_string)})
    confidence.track("navigate", {})

    value = await confidence.get_flag_string("hawkflag.color", "false")
    print(f"Flag value: {value}")


# Another asynchronous function that calls the first one
async def main():
    await get_flag()
    print("Finished calling say_hello again")
    await get_flag()
    time.sleep(5)


# Run the main function using asyncio.run (Python 3.7+)
if __name__ == "__main__":
    asyncio.run(main())
