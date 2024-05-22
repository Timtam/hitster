import React from "react"
import { CookiesProvider } from "react-cookie"
import ReactDOM from "react-dom/client"
import { HelmetProvider } from "react-helmet-async"
import { RouterProvider, createBrowserRouter } from "react-router-dom"
import "./i18n"
import "./index.css"
import Layout from "./layout"
import ErrorPage from "./pages/error-page"
import { Game, loader as GameLoader } from "./pages/game"
import { Lobby, loader as LobbyLoader } from "./pages/lobby"
import { Login, action as LoginAction } from "./pages/login"
import {
    Registration,
    action as RegistrationAction,
} from "./pages/registration"
import { Welcome, loader as WelcomeLoader } from "./pages/welcome"

const router = createBrowserRouter([
    {
        element: <Layout />,
        children: [
            {
                element: <Welcome />,
                path: "/",
                loader: WelcomeLoader,
            },
            {
                element: <Lobby />,
                path: "/lobby",
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
