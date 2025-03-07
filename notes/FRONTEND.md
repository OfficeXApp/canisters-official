# Frontend Clients

Frontend Clients are Offline-PWAs (offline progressive web apps). They support:

- service worker enables offline usage
- multi-organizations (all local browser resources are prefixed, eg. org1-cache, org1-cookies, org1-indexdb)
- each organization is a sovereign drive
- single-user that enters multiple organizations
- browser js also includes a built-in browser cache organization (own drive) called "Anonymous"
- so by default, a new visitor only see the "Anonymous" Organization
- visitor can add a cloud org, and be able to switch to it
- to persist list of users organizations, they can create a personal organization, save every other org as a drive record. then on UI there can be a button "Visit Org"
- Determine if we should support multi-users ontop of multiple orgs
