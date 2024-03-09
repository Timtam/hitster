import { HelmetProvider } from "react-helmet-async"
import React from "react"
import ReactDOM from "react-dom/client"
import App from "./App.tsx"
import { PrimeReactProvider } from "primereact/api"
import "./index.css"

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <HelmetProvider>
            <PrimeReactProvider>
                <App />
            </PrimeReactProvider>
        </HelmetProvider>
    </React.StrictMode>,
)
