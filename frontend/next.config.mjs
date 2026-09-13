const isGitHubPages = process.env.GITHUB_PAGES === "true";
const repositoryName =
  process.env.GITHUB_PAGES_REPOSITORY_NAME ?? "postal_converter_ja";

/** @type {import('next').NextConfig} */
const nextConfig = isGitHubPages
  ? {
      output: "export",
      basePath: `/${repositoryName}`,
      assetPrefix: `/${repositoryName}/`,
      images: {
        unoptimized: true,
      },
      trailingSlash: true,
    }
  : {};

export default nextConfig;
