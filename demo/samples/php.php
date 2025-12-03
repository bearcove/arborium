<?php

class SessionGuard implements StatefulGuard, SupportsBasicAuth
{
    use GuardHelpers, Macroable;

    /**
     * The name of the guard. Typically "web".
     *
     * Corresponds to guard name in authentication configuration.
     *
     * @var string
     */
    public readonly string $name;

    /**
     * The user we last attempted to retrieve.
     *
     * @var \Illuminate\Contracts\Auth\Authenticatable
     */
    protected $lastAttempted;

    /**
     * Indicates if the user was authenticated via a recaller cookie.
     *
     * @var bool
     */
    protected $viaRemember = false;

    /**
     * The number of minutes that the "remember me" cookie should be valid for.
     *
     * @var int
     */
    protected $rememberDuration = 576000;

    /**
     * The session used by the guard.
     *
     * @var \Illuminate\Contracts\Session\Session
     */
    protected $session;

    /**
     * The Illuminate cookie creator service.
     *
     * @var \Illuminate\Contracts\Cookie\QueueingFactory
     */
    protected $cookie;

    /**
     * The request instance.
     *
     * @var \Symfony\Component\HttpFoundation\Request
     */
    protected $request;

    /**
     * The event dispatcher instance.
     *
     * @var \Illuminate\Contracts\Events\Dispatcher
     */
    protected $events;

    /**
     * The timebox instance.
     *
     * @var \Illuminate\Support\Timebox
     */
    protected $timebox;

    /**
     * The number of microseconds that the timebox should wait for.
     *
     * @var int
     */
    protected $timeboxDuration;

    /**
     * Indicates if passwords should be rehashed on login if needed.
     *
     * @var bool
     */
    protected $rehashOnLogin;

    /**
     * Indicates if the logout method has been called.
     *
     * @var bool
     */
    protected $loggedOut = false;

    /**
     * Indicates if a token user retrieval has been attempted.
     *
     * @var bool
     */
    protected $recallAttempted = false;

    /**
     * Create a new authentication guard.
     *
     * @param  string  $name
     * @param  \Illuminate\Contracts\Auth\UserProvider  $provider
     * @param  \Illuminate\Contracts\Session\Session  $session
     * @param  \Symfony\Component\HttpFoundation\Request|null  $request
     * @param  \Illuminate\Support\Timebox|null  $timebox
     * @param  bool  $rehashOnLogin
     * @param  int  $timeboxDuration
     * @return void
     */
    public function __construct(
        $name,
        UserProvider $provider,
        Session $session,
        ?Request $request = null,
        ?Timebox $timebox = null,
        bool $rehashOnLogin = true,
        int $timeboxDuration = 200000,
    ) {
        $this->name = $name;
        $this->session = $session;
        $this->request = $request;
