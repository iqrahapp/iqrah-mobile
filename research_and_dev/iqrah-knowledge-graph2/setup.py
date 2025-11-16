from setuptools import setup, find_packages, find_namespace_packages

packages = find_namespace_packages(
    where="src", include=["iqrah*", "iqrah_cli*"]  # Explicitly include both packages
)

setup(
    name="iqrah",
    version="0.1.1",
    python_requires=">=3.10",
    packages=packages,
    package_dir={"": "src"},
    install_requires=[
        "httpx",
        "pydantic",
        "nest-asyncio",
        "loguru",
        "diskcache",
        "tqdm",
        "tenacity",
        "dash",
        "plotly",
        "networkx",
        "numpy",
        "beautifulsoup4",
    ],
    entry_points={
        "console_scripts": [
            "iqrah=iqrah_cli.main:main",
        ],
    },
)
