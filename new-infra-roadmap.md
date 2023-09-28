Front End
  - Applications Page

    - [x] Filter Applications
    - [x] Search Applications
    - [x] Applications show be visible for all users(logged and not logged in)

  - Application Page

    - [x] Trigger an Application
    - [x] Propose an Application
    - [x] Approve an application
    - [ ] Refill an application
    - [ ] Remove an application

  - Integrations

    - [x] Github login
    - [ ] Filecoin Wallet

Github
  - [x] Run a github action if an application issue is created, and
    convert it to a PR application

  - Run the following actions on each PR commit:

    - [ ] Check if an application was triggered
    - [ ] Check if an application was proposed
    - [ ] Check if an application was approved
    - [ ] Check if an application was refilled
    - [ ] Check if an application was removed

  - Automate merging process

    - [ ] If an application is approved
    - [ ] If an application is refilled
    - [ ] If an application is removed

Backend

  - [x] HTTP endpoints for FE(s) to consume Fil+ data

  - [ ] HTTP endpoints to register logs

  - [x] Integration with blockchain data

  - CLI tools to:

    - [ ] Convert issue to PR
    - [ ] Validate schema
    - [ ] Validate application step(s)

SSA Bot(current\not automated\ version)

  - [x] Run a cron job every one hour

  - [x] Check which of of the approved applications need refill

  - [ ] Trigger new refill PR where needed

New Repos
  - Filplus Backend

    - [ ] Open Source
    - [x] CI
    - [x] CD
  - Filplus Registry

    - [ ] Open Source
    - [x] CI
    - [x] CD
  - Filplus tools(test env)

    - [ ] Open Source
    - [x] CI
    - [x] CD
  - Filplus utils

    - [ ] Open Source
    - [x] CI
    - [x] CD
  - Filplus SSA Bot

    - [ ] Open Source
    - [ ] CI
    - [ ] CD

Test Env

  - [ ] Local test env setup
  - [ ] Staging test env setup
