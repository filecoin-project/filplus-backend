# Fil+ Backend

### About

The Fil+ Backend is a web service aiming to provide several HTTP
endpoints to manage LDN applications, perform actions on filecoin
github and read blockchain data related to applications.

### Architecture

The backend has two different external services:

* Github:
    used to manage applications.
* Blockchain data(using demob): used to validate data related to the
  different lifecycles of the application.

These services are the backbone for the core code that will be
modifying the applications, this code is located under "core" folder
and it is basically defining the structure of the json file, and a way
of changing the file.

Another important section is the `parsers.rs` this file include the
parsers that validate the structure and format of the json file.

###### Application Version
TBD


### Related Projects
- [Fil+ Registry](https://github.com/filecoin-project/filplus-registry)
- [Fil+ SSA Bot](https://github.com/filecoin-project/filplus-ssa-bot)
- [Fil+ Application Repository (Falcon)](https://github.com/filecoin-project/filecoin-plus-falcon)

### Swagger Documentation

https://app.swaggerhub.com/apis/jesraa/FilecoinBackend/1.0.0#/

### How should I use this?
There are two different kinds of endpoints:
* `/application`: these endpoints are util to manage the application.
  you can start by making a POST method to `/application` with the
  application id(see swagger documentation for detailed api
  documentation). Currently the application id is the github issue.
  after creating application via the endpoint, a new pull request will
  be created with a json file with initial data. Next step for the
  application is to be reviewed by governance team. once they are
  happy with it they can hit `/application/{id}/trigger ` to move the
  application to proposal state.
  Next, a notary should review the application and sign it with their
  ledger. In order for the notary to document the signature, they call
  `/application/{id}/propose` with data related to the blockchain tx
  made. Next step is the same as the one before, but you would call
  `/application/{id}/approve`. In that stage, the application is
  granted and completed. in order to merge the pull request you can
  call `/application/{id}/merge`.

* `/blockchain`: these endpoints retrive blockchain data related to
  ldn applications. it is using demob as a data source.

### Run Localy

- Install Rust
- Add env variables (an example present in the repo)
- `cargo run` 

### Contributions
As an open-source project, we welcome and encourage the community to contribute to the Fil+ Backend. Your insights and improvements are valuable to us. Here's how you can contribute:

- **Fork the Repository**: Start by forking the repository to your GitHub account.
- **Clone the Forked Repository**: Clone it to your local machine for development purposes.
- **Create a New Branch**: Always create a new branch for your changes.
- **Make Your Changes**: Implement your features, bug fixes, or improvements.
- **Commit Your Changes**: Make sure to write clear, concise commit messages.
- **Push to Your Fork**: Push your changes to your forked repository.
- **Create a Pull Request**: Submit a pull request from your forked repository to our main repository.

Please read through our [CONTRIBUTING.md](CONTRIBUTING.md) file for detailed instructions on how to contribute.

### License
This project is dual-licensed under the `Permissive License Stack`, which means you can choose to use the project under either:

- The Apache License 2.0, which is a free and open-source license, focusing on patent rights and copyright notices. For more details, see the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).

- The MIT License, a permissive and open-source license, known for its broad permissions and limited restrictions. For more details, see the [MIT License](https://opensource.org/licenses/MIT).

You may not use the contents of this repository except in compliance with one of these licenses. For an extended clarification of the intent behind the choice of licensing, please refer to the `LICENSE` file in this repository or visit [Permissive License Stack Explanation](https://protocol.ai/blog/announcing-the-permissive-license-stack/).

For the full license text, please see the [LICENSE](LICENSE) file in this repository.

### CHANGELOG
1.0.4
- The database workspace is temporarily removed from the project and as a dependency.