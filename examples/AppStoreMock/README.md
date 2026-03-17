# AppStoreMock

This directory contains a simulated iOS project structure designed to test App Store submission rules.

## Structure

-   `ProductionApp.xcodeproj/`: Mock Xcode project.
-   `ProductionApp.xcworkspace/`: Mock Xcode workspace.
-   `Distribution/ProductionApp.app/`: The compiled bundle containing:
    -   `Info.plist`: Declares permissions (Camera, Location).
    -   `PrivacyInfo.xcprivacy`: Declares data collection categories.
    -   `Frameworks/`: Contains mock SDKs (FirebaseCore, GoogleAnalytics) for SDK cross-check rules.
    -   `embedded.mobileprovision`: Mock provisioning profile.

## How to Test

1.  Open the verifyOS web frontend.
2.  Click **"Choose folder"**.
3.  Select this `AppStoreMock` folder.
4.  The tool will automatically discover the project and the app bundle.
5.  Select a target and click **"Run scan"**.
