from shared import Confidence
import asyncio

async def get_flag():
    confidence = Confidence()
    value = await confidence.get_flag_string("hawkflag.message", "false")
    print(f"Flag value: {value}")

# Another asynchronous function that calls the first one
async def main():
    print("Calling say_hello...")
    await get_flag()
    print("Finished calling say_hello")
    await get_flag()
    print("again")

# Run the main function using asyncio.run (Python 3.7+)
if __name__ == "__main__":
    asyncio.run(main())

