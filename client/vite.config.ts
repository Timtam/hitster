import gitVersion from "@corteks/gitversion"
import react from "@vitejs/plugin-react-swc"
import { defineConfig, splitVendorChunkPlugin } from "vite"
import checker from "vite-plugin-checker"
import replace from "vite-plugin-filter-replace"
import viteTsconfigPaths from "vite-tsconfig-paths"

// https://vitejs.dev/config/
export default defineConfig(async () => {
    let gv = await gitVersion.default()
    let branch =
        (process.env.HITSTER_BRANCH ?? gv.CURRENT_BRANCH) || "development"
    let version: string = process.env.HITSTER_VERSION

    if (!version) {
        if (!gv.COMMITS_SINCE_TAG) version = gv.LAST_TAG_NAME ?? ""
        else
            version = `${gv.LAST_TAG_NAME}+${gv.COMMITS_SINCE_TAG}-${gv.CURRENT_COMMIT_SHORT_ID}`
    }

    if (!version) {
        // fallback
        version = "unknown"
    }

    return {
        plugins: [
            react(),
            viteTsconfigPaths(),
            checker({
                typescript: true,
            }),
            splitVendorChunkPlugin(),
            replace([
                {
                    filter: "src/navigation.tsx",
                    replace: [
                        {
                            from: /__VERSION__/g,
                            to: version,
                        },
                        {
                            from: /__BRANCH__/g,
                            to: branch,
                        },
                    ],
                },
            ]),
        ],
    }
})
