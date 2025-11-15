# src/iqrah_cli/main.py

import argparse
import sys
from .commands import build, visualize


def main():
    parser = argparse.ArgumentParser(
        description="Iqrah - Quranic Knowledge Graph Tools"
    )
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Add build command
    build.setup_parser(subparsers)

    # Add visualize command
    visualize.setup_parser(subparsers)

    args = parser.parse_args()

    if args.command is None:
        parser.print_help()
        sys.exit(1)

    try:
        if args.command == "build":
            build.run(args)
        elif args.command == "visualize":
            visualize.run(args)
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
