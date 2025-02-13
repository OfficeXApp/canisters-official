# Permissions ACL

Note that we have directory permissions (for files & folders) and system permissions (for drive crud).

## Directory Permissions

I have attached a loose reference of how i would like my permission system to work. Here is also a text summary:
-i have a directory structure of files and folders

-i want to grant permissions on files and folders (directory resources)

-i have users in teams

-i want to grant permissions on resources to users or teams

-the permissions have a start and end date

-the permissions on directory resources are view, upload (upload means a user can also edit and delete files they uploaded), edit (edit does not include delete), delete (delete includes edit but not upload), webhooks (webhooks means they can set webhooks), manage (manage lets users grant or modify permissions, as well as edit upload delete webhooks, its the most permissive)

-managers can only grant permissions equal or less to what permissions they have

-teams have owner, admin, member. all admins are also members. owner and admins can invite members. only owner can manage admins.

-there is a concept of a one-time link which can be converted into a permission. anyone that can create permissions can create one-time links.

-we must be able to quickly view a resources permissions as well as one-time links

-dont worry about advanced acl on teams, keep it simple as we already have an invite system that doesnt require accept/reject invites.

- i want permissions of a folder to be inheritable to subfolders, but i also want files or folders to be able to have "sovereign permissions" where it isnt influenced by parent folder permissions. we can update the file and folder types to enable this if need. managers can still modify the soverign permission but uploaders cannot set soverign on their own files.

## System Permissions
