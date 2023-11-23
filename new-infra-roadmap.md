Front End
  - Applications Page

    - [x] Filter Applications
    - [x] Search Applications
    - [x] Applications show be visible for all users(logged and not logged in)

  - Application Page

    - [x] Trigger an Application
    - [x] Propose an Application
    - [x] Approve an application
    - [x] Refill an application

  - Integrations

    - [x] Github login
    - [x] Filecoin Wallet

Github
  - [x] Create new issue template
  - [x] Finalize schema file
  - [x] Run a github action if an application issue is created, and
    convert it to a PR application

  - Run the following actions on each PR commit:

    - [x] Check if an application was triggered
    - [x] Check if an application was proposed
    - [x] Check if an application was approved
    - [x] Check if an application was refilled
    - [ ] Check if an application was removed

  - Automate merging process

    - [x] If an application is approved
    - [x] If an application is refilled
    - [ ] If an application is removed

Backend

  - [x] HTTP endpoints for FE(s) to consume Fil+ data

  - [x] Integration with blockchain data

SSA Bot(current\not automated\ version)

  - [x] Run a cron job every one hour

  - [x] Check which of of the approved applications need refill

  - [x] Trigger new refill PR where needed

New Repos
  - Filplus Backend

    - [ ] Open Source (first, invalidate current gh private key as its exposed and generate new one)
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

