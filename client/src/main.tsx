import { setUseWhatChange } from "@simbathesailor/use-what-changed"
import "bootstrap/dist/css/bootstrap.min.css"
import React from "react"
import { ToastsProvider } from "react-bootstrap-toasts"
import { CookiesProvider } from "react-cookie"
import ReactDOM from "react-dom/client"
import { HelmetProvider } from "react-helmet-async"
import { RouterProvider, createBrowserRouter } from "react-router-dom"
import "./i18n"
import "./index.css"
import Layout from "./layout"
import ErrorPage from "./pages/error-page"
import GameLoader from "./pages/game.loader"
import Game from "./pages/game.page"
import LobbyLoader from "./pages/lobby.loader"
import Lobby from "./pages/lobby.page"
import LoginAction from "./pages/login.action"
import Login from "./pages/login.page"
import RegistrationAction from "./pages/registration.action"
import Registration from "./pages/registration.page"

setUseWhatChange(import.meta.env.DEV)

const router = createBrowserRouter(
    [
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
            ],
            errorElement: <ErrorPage />,
        },
    ],
    {
        future: {
            v7_relativeSplatPath: true,
        },
    },
)

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
                    <RouterProvider
                        future={{ v7_startTransition: true }}
                        router={router}
                    />
                </ToastsProvider>
            </CookiesProvider>
        </HelmetProvider>
    </React.StrictMode>,
)
