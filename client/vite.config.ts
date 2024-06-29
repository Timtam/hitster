import react from "@vitejs/plugin-react-swc"
import fs from "fs"
import toml from "toml"
import { defineConfig, splitVendorChunkPlugin } from "vite"
import checker from "vite-plugin-checker"
import replace from "vite-plugin-filter-replace"
import viteTsconfigPaths from "vite-tsconfig-paths"
import pkg from "./package.json"

let server_version: string = "UNKNOWN"

try {
    server_version = toml.parse(
        fs.readFileSync("../server/Cargo.toml", { encoding: "utf-8" }),
    ).package.version
} catch (e) {
    server_version = toml.parse(
        fs.readFileSync("./Cargo.toml", { encoding: "utf-8" }),
    ).package.version
}

// https://vitejs.dev/config/
export default defineConfig({
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
                        from: /__CLIENT_VERSION__/g,
                        to: pkg.version,
                    },
                    {
                        from: /__SERVER_VERSION__/g,
                        to: server_version,
                    },
                ],
            },
        ]),
    ],
})
