module Page.Article exposing (Model, Msg, init, subscriptions, toSession, update, view)

{-| Viewing an individual article.
-}

import Api exposing (Cred)
import Api.Endpoint as Endpoint
import Article exposing (Article, Full, Preview)
import Article.Body exposing (Body)
import Article.Comment as Comment exposing (Comment)
import Article.Slug as Slug exposing (Slug)
import Author exposing (Author(..), FollowedAuthor, UnfollowedAuthor)
import Avatar
import Browser.Navigation as Nav
import CommentId exposing (CommentId)
import Html exposing (..)
import Html.Attributes exposing (attribute, class, disabled, href, id, placeholder, value)
import Html.Events exposing (onClick, onInput, onSubmit)
import Http
import Json.Decode as Decode
import Loading
import Log
import Page
import Profile exposing (Profile)
import Route
import Session exposing (Session)
import Task exposing (Task)
import Time
import Timestamp
import Username exposing (Username)
import Viewer exposing (Viewer)



-- MODEL


type alias Model =
    { session : Session
    , timeZone : Time.Zone
    , errors : List String

    -- Loaded independently from server
    , comments : Status ( CommentText, List Comment )
    , article : Status (Article Full)
    }


type Status a
    = Loading
    | LoadingSlowly
    | Loaded a
    | Failed


type CommentText
    = Editing String
    | Sending String


init : Session -> Slug -> ( Model, Cmd Msg )
init session slug =
    let
        maybeCred =
            Session.cred session
    in
    ( { session = session
      , timeZone = Time.utc
      , errors = []
      , comments = Loading
      , article = Loading
      }
    , Cmd.batch
        [ Article.fetch maybeCred slug
            |> Http.send CompletedLoadArticle
        , Comment.list maybeCred slug
            |> Http.send CompletedLoadComments
        , Task.perform GotTimeZone Time.here
        , Task.perform (\_ -> PassedSlowLoadThreshold) Loading.slowThreshold
        ]
    )



-- VIEW


view : Model -> { title : String, content : Html Msg }
view model =
    case model.article of
        Loaded article ->
            let
                { title } =
                    Article.metadata article

                author =
                    Article.author article

                avatar =
                    Profile.avatar (Author.profile author)

                slug =
                    Article.slug article

                profile =
                    Author.profile author

                buttons =
                    case Session.cred model.session of
                        Just cred ->
                            viewButtons cred article author

                        Nothing ->
                            []
            in
            { title = title
            , content =
                div [ class "article-page" ]
                    [ div [ class "banner" ]
                        [ div [ class "container" ]
