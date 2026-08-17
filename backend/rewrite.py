import re

with open("src/api/storage.rs", "r") as f:
    api_content = f.read()

# I will write a simple parser to extract the bodies of the endpoints and move them to service.
# Wait, parsing Rust with regex is very brittle.
