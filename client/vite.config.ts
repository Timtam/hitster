import { getFormattedVersion } from "@corteks/gitversion"
import react from "@vitejs/plugin-react-swc"
import { defineConfig, splitVendorChunkPlugin } from "vite"
import checker from "vite-plugin-checker"
import replace from "vite-plugin-filter-replace"
import viteTsconfigPaths from "vite-tsconfig-paths"

// https://vitejs.dev/config/
export default defineConfig(async () => {
    let version: string = process.env.HITSTER_VERSION ?? await getFormattedVersion()

    if(version === "0.0.0-null") {
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
                    ],
                },
            ]),
        ],
    }
})
