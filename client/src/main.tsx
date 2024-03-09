import { HelmetProvider } from "react-helmet-async"
import ErrorPage from "./error-page"
import Layout from "./layout"
import React from "react"
import ReactDOM from "react-dom/client"
import { Lobby, loader as LobbyLoader } from "./lobby"
import { PrimeReactProvider } from "primereact/api"
import "./index.css"
import { createBrowserRouter, RouterProvider } from "react-router-dom"

const router = createBrowserRouter([
    {
        element: <Layout />,
        children: [
            {
                element: <Lobby />,
                path: "/",
                loader: LobbyLoader,
            },
        ],
        errorElement: <ErrorPage />,
    },
])

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <HelmetProvider>
            <PrimeReactProvider>
                <RouterProvider router={router} />
            </PrimeReactProvider>
        </HelmetProvider>
    </React.StrictMode>,
)
