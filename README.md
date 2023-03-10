
### Release cutting
Creating a release on your template will trigger the release pipeline, which packages [download bundles]((https://github.com/kurbos/bevy-shell-template/releases/latest)) for all major platforms, including mobile and web. The pipeline will create a branch `gh-pages` with the WASM bundle to serve by GitHub Pages ([demo](https://kurbos.github.io/bevy-shell-template)), or DockerHub image ([example](https://hub.docker.com/repository/docker/simbleau/my_game)), depending on the [hosting strategy](#-hosting) you choose to setup.

> 🔥 **WARNING: We enforce releases are tagged with a semantic version name**, e.g. "*v0.1.0*", not "*v1*"
> This can be modified on the [`release-*` workflow files](.github/workflows/).

## 📡 Web Hosting
There are two ways to host the WASM build of your Bevy game, with Docker or GitHub Pages. You could be creative to adapt this to other hosting platforms, but we will only explain these two. You would likely choose one, not both. If you don't have hosting equipment or know what you're doing, choose GitHub Pages.

### GitHub Pages
To automatically serve your WASM bundle like [our demo](https://kurbos.github.io/bevy-shell-template/), here are the steps:
- Modify the [GitHub Pages GitHub Action file](.github/workflows/release-gh-pages.yml)'s variarable `PUBLIC_URL` with the slug for your GitHub Pages hosting.
  - If the repo name is the same as the repo owner, this should be `/`, otherwise, it will should be `/<repository-name>/` (e.g. `/bevy-shell-template/`)
- *Optional*: Delete the [DockerHub GitHub Action](.github/workflows/release-dockerhub.yml), as you probably don't need it.
- [Cut a release](#release-cutting) and wait for pipeline completion
- On your GitHub template repo, visit Settings > Pages
- Select `gh-pages` branch from the dropdown menu and press "Save".

  <img src="https://user-images.githubusercontent.com/20546772/184507297-e0f7ff46-57e6-4329-9a79-f2d5ceb5d97a.png" width="600" height="auto"/>

## 🚀 Launchers
### WASM (Web)
To build and run the WASM app locally we recommend [Trunk](https://trunkrs.dev/):
> Serve with `trunk serve` and open [`http://127.0.0.1:8080`](http://127.0.0.1:8080) in your browser
- Assets are streamed through the hosting provider, so that the initial WASM bundle is smaller.
- We use all the WASM optimizations discussed described [here](https://rustwasm.github.io/book/reference/code-size.html) in the Rust and WebAssembly book.
- There is an initial loading screen provided through [Yew](https://yew.rs) while the WASM bundle loads.
