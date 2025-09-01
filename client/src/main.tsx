import { HelmetProvider } from "@dr.pogodin/react-helmet"
import "bootstrap/dist/css/bootstrap.min.css"
import React from "react"
import { CookiesProvider } from "react-cookie"
import ReactDOM from "react-dom/client"
import { RouterProvider, createBrowserRouter } from "react-router"
import "./i18n"
import "./index.css"
import Layout from "./layout"
import BrowserLoader from "./pages/browser.loader"
import Browser from "./pages/browser.page"
import ErrorPage from "./pages/error-page"
import GameLoader from "./pages/game.loader"
import Game from "./pages/game.page"
import HitLoader from "./pages/hit.loader"
import Hit from "./pages/hit.page"
import LobbyLoader from "./pages/lobby.loader"
import Lobby from "./pages/lobby.page"
import LoginAction from "./pages/login.action"
import Login from "./pages/login.page"
import RegistrationAction from "./pages/registration.action"
import Registration from "./pages/registration.page"
import { ToastsProvider } from "./toasts"

const router = createBrowserRouter([
    {
        hydrateFallbackElement: <p>Loading...</p>,
        element: <Layout />,
        children: [
            {
                element: <Lobby />,
                path: "/",
                loader: LobbyLoader,
            },
            {
                element: <Browser />,
                path: "/hits",
                loader: BrowserLoader,
            },
            {
                element: <Registration />,
                path: "/register",
                action: RegistrationAction,
            },
            {
                element: <Login />,
                path: "/login",
                action: LoginAction,
            },
            {
                element: <Game />,
                path: "/game/:gameId",
                loader: GameLoader,
            },
            {
                element: <Hit />,
                path: "/hits/:hitId",
                loader: HitLoader,
            },
        ],
        errorElement: <ErrorPage />,
    },
])

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <HelmetProvider>
            <CookiesProvider defaultSetOptions={{ path: "/" }}>
                <ToastsProvider
                    toastContainerProps={{
                        position: "top-end",
                        className: "p-3",
                        "aria-hidden": true,
                    }}
                >
                    <RouterProvider router={router} />
                </ToastsProvider>
            </CookiesProvider>
        </HelmetProvider>
    </React.StrictMode>,
)
