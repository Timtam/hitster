<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![GPLv3 License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/Timtam/hitster">
    <!-- <img src="images/logo.png" alt="Logo" width="80" height="80"> -->
    Hitster on GitHub
  </a>

<h3 align="center">Hitster Online</h3>

  <p align="center">
    An unofficial web-based implementation of the <a href="https://hitstergame.com/">Hitster card game by Koninklijke Jumbo B.V.</a>
    <br />
    <br />
    <a href="https://hitster.toni-barth.online/">View Stable Demo</a>
    ·
    <a href="https://hitster-dev.toni-barth.online/">View Development Demo</a>
    ·
    <a href="https://github.com/Timtam/hitster/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    ·
    <a href="https://github.com/Timtam/hitster/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#docker">Docker</a></li>
        <ul>
          <li><a href="#launching-a-container">Launching a container</a></li>
          <li><a href="#volumes">Volumes</a></li>
          <li><a href="#docker-compose">Docker Compose</a></li>
        </ul>
        <li><a href="#local">Local</a></li>
        <ul>
          <li><a href="#prerequisites">Prerequisites</a></li>
          <li><a href="#building">Building</a></li>
        </ul>
        <li><a href="#attention">ATTENTION</a></li>
        <li><a href="#environment-variables">Environment Variables</a></li>
      </ul>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

<!-- [![Product Name Screen Shot][product-screenshot]](https://example.com) -->

Hitster is a music quiz card game developed and released by Koninklijke Jumbo B.V. Its very easy to play and is fun to play for literally everyone. Here is a short rundown of how to play:

* Everyone receives a hit card at the beginning of a game. A hit contains information about a song, containing its title, the artist and year when it was released.
* A short snippet of a hit is played to you. You'll have to guess if it was released either before or after the hit that you already have in your collection.
* If you guessed correctly, you'll earn the hit card and add it to your collection. The game will continue to the next player.
* Next time it's your turn, you'll be played a hit again, but this time, you'll have to guess if it was released either before your earliest hit's release year, between your two hits, or after the latest hit release. Guess correctly to earn yourself another hit card, grow your collection, but also make it harder to guess your next hit correctly.

There is more to it, like tokens you can earn by also guessing title and artist of a hit, and paying them to intercept your opponents by correcting their guesses to earn their hit for your own. But see for yourself, you don't need to register, just visit the demo and play a game with at least one of your friends.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* [![React][React.js]][React-url]
* [![React-Bootstrap][Bootstrap.com]][Bootstrap-url]
* [Rocket][rocket-url]

... and loads more

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

Hitster consists of multiple separate projects:

* a server component deploying a REST API, written in Rust and based on Rocket
* a client application responsible for displaying the game's UI and interacting with the server, written in React and TypeScript
* a cli helper to run some tasks that aren't necessary to have within the server itself

Follow these steps to get a dev environment ready to run the project locally.

### Docker

#### Launching a container

The project is supposed to run inside a Docker container. As such, it holds a Dockerfile which builds the project. It also pushes docker images whenever a commit is created. If you just want to spin it up locally to give it a try, run the following command with Docker installed:

```sh
docker run -p 8000:8000 tonironaldbarth/hitster:latest
```

Multiple containers are available to choose from:

| tag | purpose |
| --- | ------- |
| latest | newest stable version |
| dev | bleeding edge dev (might break unpredictably, contains all newest features that haven't been merged into stable) |
| <release_name> | a specific release that won't change anymore, see the [list of available tags](https://github.com/Timtam/hitster/tags) for your possible options |

The Docker containers currently only automatically gets built for amd64 and arm64, feel free to open an issue or pull request to add/request more build targets.

If you want to build the container for yourself, clone the repository, open a command line and navigate into the project directory, then run the following commands:

```sh
docker build -t hitster .
docker run -p 8000:8000 hitster
```

The Hitster application will be accessible on localhost Port 8000 afterwards. Please see below for a <a href="#environment-variables">list of environment variables</a> to further configure the container.

#### Volumes

The Docker container exposes some volumes which you can connect to to persist your data. The following table holds information about the available paths:

| Path | Description |
| ---- | ----------- |
| /hitster.sqlite | the database holding registered user information etc |
| /hits | the folder where downloaded hits are stored |

You can launch a docker container by specifying your volumes like follows:

```sh
docker run -v hits:/hits -v hitster.sqlite:/hitster.sqlite -p 8000:8000 tonironaldbarth/hitster
```

#### Docker Compose

An easier way of launching a Hitster server is by utilizing Docker Compose. You can find an example docker-compose.yml file below:

```yaml
services:
  hitster:
    image: tonironaldbarth/hitster:latest
    ### comment above line and uncomment if you want to build from source
    #build:
    #  context: .
    restart: unless-stopped
    volumes:
      - ./hitster.sqlite:/hitster.sqlite
      - ./hits:/hits
    environment:
      - ROCKET_SECRET_KEY=generate_me_a_secret_key
      - ROCKET_ADDRESS=0.0.0.0
      - ALTCHA_KEY=GENERATE_ME_ANOTHER_SECRET_KEY
    ports:
      - "8000:8000"
```

### Local

Setting up the project locally will requires several tools, which need to be accessible on your PATH environment variable to be called without having to know their exact path. We strictly recommend to stick to a Docker-based development environment instead. Should you want to still go for a local setup, please find the instructions below.

#### Prerequisites

You'll need the following tools to be installed:

* Node.js v20
* Rust: we recommend to install the most recent stable version, that'll most likely be the version we're developing with as well
* FFMpeg: we recommend installing the yt-dlp compatible static builds from [its corresponding GitHub repository](https://github.com/yt-dlp/FFmpeg-Builds)
* [FFMpeg-normalize](https://github.com/slhck/ffmpeg-normalize)
* (optional) [yt-dlp](https://github.com/yt-dlp/yt-dlp). Make sure to <a href="#yt-dlp">check the section on yt-dlp</a> to enable the usage of yt-dlp (disabled by default)

Ensure that everything is working by running the following test commands and ensuring proper output:

```sh
node -v
rustup show
ffmpeg -version
ffmpeg-normalize
yt-dlp -v
```

#### Building

Start by cloning this repository. Open a command line and navigate into the folder of the cloned repository. Afterwards, run those commands to build all components of the Hitster project:

* client:
  ```sh
  cd client
  npm install
  npm run build
  cd ..
  ```
* cli:
  ```sh
  cargo build -p hitster-cli
  ```
* server:
  ```sh
  cargo run
  ```
  
Please note that you'll need to specify a certain set of environment variables when running the application locally in order for it to start. You can find <a href="#environment-variables">the list of environment variables</a> below.
  
### Creating the hitster database

When trying to run the server, you might ask yourself, how do I get this hitster.sqlite file everyone is talking about? You might be seeing errors like these:

```sh
error: error returned from database: (code: 14) unable to open database file
```

All you need to do to fix this is provide an empty file, the server will populate all the necessary database info it needs. To do this, just create an empty text file and call it hitster.sqlite, or if on a Unix-based command line, use:

```sh
touch hitster.sqlite
```

### Creating an administrator account

Chances are high you won't ever need an administrator account, as the game can be played without even registering a user. If you want to create your own packs or hits though, or fix already existing ones, you'll need to have at least one administrator account registered. You can easily do that with the help of the hitster cli tool.

#### Local

When running Hitster locally, run the following command:

```sh
cargo run -p hitster-cli -- users create -a <username>
```

Replace <username\> with the username of choice. You'll be prompted to input a password and your new user will be created. You should be able to login via the web interface and use this account to make changes to the Hitster database.

#### Docker

When running Hitster in Docker, you can run the following command to create a new administrative user:

```sh
docker run -v hitster.sqlite:/hitster.sqlite --entrypoint /hitster/cli tonironaldbarth/hitster:latest users create -a <username>
```

If running in Docker Compose, the command would look something like this:

```sh
docker compose run --entrypoint /hitster/cli hitster users create -a <username>
```

Replace <username\> with the username of choice. You'll be prompted to input a password and your new user will be created. You should be able to login via the web interface and use this account to make changes to the Hitster database.

### yt-dlp

By default, the Hitster server will try to download songs from YouTube with the help of a rust-native library called rusty_ytdl. This requires less dependencies to be running alongside the Hitster server and thus is preferred when setting up Hitster locally. It however is also less reliable as it runs into YouTube blocking mechanisms more frequently. If you therefore want to use yt-dlp instead, you'll need to build the Hitster server with the yt_dl feature enabled. Navigate into the server directory and run:

```sh
cargo run --features yt_dl
```

This will enable yt-dlp support as a fallback for the rust-native way of downloading songs from YouTube. If you want to skip the native way of downloading alltogether, you can disable default features and just enable the yt_dl feature alone, as follows:

```sh
cargo run --no-default-features --features yt_dl
```

This also is the default within the Docker container.

### Environment Variables

The project can be configured through environment variables. Environment variables can be populated in different ways, depending on how you are running it.

* (local only) by setting them via EXPORT on Linux or SET on Windows, e.g.:
  ```sh
  export DATABASE_URL=~/sqlite://hitster.sqlite
  ```
* (local only) by specifying them inside a .env file, which will have to be placed inside the server directory of this repository. A file could look like this:
  ```sh
  DATABASE_URL=sqlite://hitster.sqlite
  ```
* (Docker only) handing them to the docker run command via the -e switch, e.g.:
  ```sh
  docker run -e DATABASE_URL=sqlite://hitster.sqlite -p 8000:8000 tonironaldbarth/hitster
  ```

The following environment variables are available. Required variables are set to default values when running via docker.

| variable | required | meaning |
| -------- | -------- | ------- |
| DATABASE_URL | yes | location of the database file, must be in the format of sqlite://path_to_file.sqlite, /hitster.sqlite in Docker containers by default |
| ALTCHA_KEY | no | a random secret key necessary to generate altcha challenges, it isn't required to run the server, but it'll be required if you're planning to allow user registration and other form submissions |
| CLIENT_DIRECTORY | no | specify the location of the compiled client files, usually not needed in Docker, ./client in local mode |
| DOWNLOAD_DIRECTORY | no | download location of the songs downloaded by the server, /hits in Docker containers by default, ./hits otherwise |

In addition to those custom environment variables, the server can be further tweaked by populating Rocket-specific environment variables. Some important variables would be ROCKET_ADDRESS to specify the address to bind to the server, as well as ROCKET_PORT to change the port the server is listening on. For a permanently deployed service, we recommend setting the ROCKET_SECRET_KEY environment variable to a randomly generated key, which will allow users to stay logged in even if the server restarts. Please see the [list of rocket environment variables](https://rocket.rs/guide/v0.5/configuration/) on the rocket website.

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- ROADMAP -->
## Roadmap

See the [open issues](https://github.com/github_username/repo_name/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the GNU General Public License, version 3. See `LICENSE` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Toni Barth - [ToniBarth@troet.cafe](https://troet.cafe/@tonibarth) - contact@toni-barth.online

Project Link: <https://github.com/Timtam/hitster>

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/Timtam/hitster.svg?style=for-the-badge
[contributors-url]: https://github.com/Timtam/hitster/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Timtam/hitster.svg?style=for-the-badge
[forks-url]: https://github.com/Timtam/hitster/network/members
[stars-shield]: https://img.shields.io/github/stars/Timtam/hitster.svg?style=for-the-badge
[stars-url]: https://github.com/Timtam/hitster/stargazers
[issues-shield]: https://img.shields.io/github/issues/Timtam/hitster.svg?style=for-the-badge
[issues-url]: https://github.com/Timtam/hitster/issues
[license-shield]: https://img.shields.io/github/license/Timtam/hitster.svg?style=for-the-badge
[license-url]: https://github.com/Timtam/hitster/blob/master/LICENSE
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://www.linkedin.com/in/toni-barth-a54071174/
[product-screenshot]: images/screenshot.png
[React.js]: https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB
[React-url]: https://reactjs.org/
[Bootstrap.com]: https://img.shields.io/badge/Bootstrap-563D7C?style=for-the-badge&logo=bootstrap&logoColor=white
[Bootstrap-url]: https://getbootstrap.com
[Rocket-url]: https://rocket.rs/
