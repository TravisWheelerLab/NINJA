from typing import List
import random
import sys

if len(sys.argv) != 3:
    print("Usage: python generate_benchmark.py <COUNT> <SIZE>", file=sys.stderr)
    exit(1)

SEQUENCE_COUNT = int(sys.argv[1])
SEQUENCE_SIZE = int(sys.argv[2])

# We add "-" more than once to increase its odds of being chosen
# in order to make the generated sequences a qualitatively more
# similar to actual sequences we might see.
ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ-------------"

NAME_FRAGMENTS = [
    "Adney",
    "Aldo",
    "Aleyn",
    "Alford",
    "Amherst",
    "Angel",
    "Anson",
    "Archibald",
    "Aries",
    "Arwen",
    "Astin",
    "Atley",
    "Atwell",
    "Audie",
    "Avery",
    "Ayers",
    "Baker",
    "Balder",
    "Ballentine",
    "Bardalph",
    "Barker",
    "Barric",
    "Bayard",
    "Bishop",
    "Blaan",
    "Blackburn",
    "Blade",
    "Blaine",
    "Blaze",
    "Bramwell",
    "Brant",
    "Brawley",
    "Breri",
    "Briar",
    "Brighton",
    "Broderick",
    "Bronson",
    "Bryce",
    "Burdette",
    "Burle",
    "Byrd",
    "Byron",
    "Cabal",
    "Cage",
    "Cahir",
    "Cavalon",
    "Cedar",
    "Chatillon",
    "Churchill",
    "Clachas",
    "Addison",
    "Alivia",
    "Allaya",
    "Amarie",
    "Amaris",
    "Annabeth",
    "Annalynn",
    "Araminta",
    "Ardys",
    "Ashland",
    "Avery",
    "Bedegrayne",
    "Bernadette",
    "Billie",
    "Birdee",
    "Bliss",
    "Brice",
    "Brittany",
    "Bryony",
    "Cameo",
    "Carol",
    "Chalee",
    "Christy",
    "Corky",
    "Cotovatre",
    "Courage",
    "Daelen",
    "Dana",
    "Darnell",
    "Dawn",
    "Delsie",
    "Denita",
    "Devon",
    "Devona",
    "Diamond",
    "Divinity",
    "Duff",
    "Dustin",
    "Dusty",
    "Ellen",
    "Eppie",
    "Evelyn",
    "Everilda",
    "Falynn",
    "Fanny",
    "Faren",
    "Freedom",
    "Gala",
    "Galen",
    "Gardenia",
]
NAME_FRAGMENT_COUNT = len(NAME_FRAGMENTS)

NAME_SIZE: int = 1
while NAME_FRAGMENT_COUNT ** NAME_SIZE < SEQUENCE_COUNT:
    NAME_SIZE += 1

names_generated: int = 0
sequences_generated: int = 0


def generate_name() -> str:
    global names_generated

    name_index: int = names_generated

    fragments: List[str] = []
    for _ in range(NAME_SIZE):
        fragment_index: int = name_index % NAME_FRAGMENT_COUNT
        fragment = NAME_FRAGMENTS[fragment_index]
        fragments.append(fragment)

        name_index = name_index // NAME_FRAGMENT_COUNT

    names_generated += 1
    return "-".join(fragments)


def generate_sequence() -> str:
    return "".join(random.choices(ALPHABET, k=SEQUENCE_SIZE))


if __name__ == "__main__":
    for _ in range(SEQUENCE_COUNT):
        name = generate_name()
        sequence = generate_sequence()
        print(f">{name}")
        print(sequence)
