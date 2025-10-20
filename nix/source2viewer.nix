{
  fetchFromGitHub,
  buildDotnetModule,
  dotnetCorePackages,
}:
# Builds ValveResourceFormat CLI from source
buildDotnetModule {
  pname = "ValveResourceFormat";
  version = "15.0";

  # GitHub source
  src = fetchFromGitHub {
    owner = "ValveResourceFormat";
    repo = "ValveResourceFormat";
    rev = "aa1c384b9481a6892b3963d96251b3065191db47";
    sha256 = "sha256-1p7AeiZ5Bgre77XuS5xjwjk4Kzs0spEh2e/WHLuQG5I=";
  };

  nugetDeps = ./deps.json;
  # .NET 9.0 runtime and SDK
  dotnet-sdk = dotnetCorePackages.sdk_9_0;
  dotnet-runtime = dotnetCorePackages.runtime_9_0;
  projectFile = "./CLI/CLI.csproj";
}
