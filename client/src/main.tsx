import React from "react"
import { CookiesProvider } from "react-cookie"
import ReactDOM from "react-dom/client"
import { HelmetProvider } from "react-helmet-async"
import { RouterProvider, createBrowserRouter } from "react-router-dom"
import ErrorPage from "./error-page"
import "./index.css"
import Layout from "./layout"
import { Lobby, loader as LobbyLoader } from "./lobby"
import { Registration, action as RegistrationAction } from "./registration"

const router = createBrowserRouter([
    {
        element: <Layout />,
        children: [
            {
                element: <Lobby />,
                path: "/",
                loader: LobbyLoader,
            },
            {
                element: <Registration />,
                path: "/register",
                action: RegistrationAction,
            },
        ],
        errorElement: <ErrorPage />,
    },
])

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <HelmetProvider>
            <CookiesProvider defaultSetOptions={{ path: "/" }}>
                <RouterProvider router={router} />
            </CookiesProvider>
        </HelmetProvider>
    </React.StrictMode>,
)
