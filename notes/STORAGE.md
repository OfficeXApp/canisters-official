# Storage

There are 4 types of storage:

1. BrowserCache
2. LocalSSD
3. ICP Canister
4. Cloud S3/Storj

## Uploading

(1) Browser Cache, and (2) LocalSSD "uploads" only travel to the local hardware. Downloads too.

(3) ICP Canister uploads happen in HTTP chunks and get saved as binary in stable memory of ICP canister. When downloaded, it is chunked and final file is constructed on client js app. It currently does not support a single url string that can be shared or used in html img/video tags. This is advanced functionality waiting on some improvements from Dfinity ICP core developers.

(4) Cloud S3/Storj uploads actually happen on the clientside js, instead of sending to server ICP canister relaying to S3. Bandwidth on icp canisters is expensive, so clientside js upload is much more pragmatic. To achieve this, the drive canister listens to `POST /directory/action` for file upload actions list. For each upload action, we create the file & folder metadata and insert it into hashtables, then return a presigned url to allow client to upload direct to S3 at a specific path exactly matching the expected filename and id which is simply `/files/{fileUUID}.mp4`. This inherently supports file versioning as fileUUID changes on new versions of a file. We must remember to delete the old versions when appropriate.

When users request this S3 file, our canister maintains a proxy raw_url link `officex.app/.../fileUUID.mp4` which receives GET request and canister responds with 302 redirect to the actual aws s3 raw_url with an on-the-fly temporary presigned url access file. The presigned url does expire, and thats why its the proxy raw_url that gets shared by users or used in html img/video tags. The proxy raw_url goes through the canister every time which means we can adjust ACL anytime, but also costs gas.

## Downloading

For simplicity, all downloads are to the users computer. If they want the file to be downloaded to a certain disk, they must manually upload it to the disk. Perhaps advanced functionality we can let users download direct to a disk.

## S3 Bucket CORS

When adding disk "AWS S3 Buckets", users are responsible for enabling cors! In the AWS Console GUI it can be done by navigating to `AWS S3 > YourBucket > Permissions > Cross-Origin resource sharing (CORS)` and pasting this permissive cors policy:

```json
[
  {
    "AllowedHeaders": ["*"],
    "AllowedMethods": ["GET", "POST", "PUT"],
    "AllowedOrigins": ["*"],
    "ExposeHeaders": []
  }
]
```
