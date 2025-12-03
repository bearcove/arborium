(defpackage lack/middleware/session
  (:nicknames :lack.middleware.session)
  (:use :cl)
  (:import-from :lack/session/store
                :fetch-session
                :store-session
                :remove-session)
  (:import-from :lack/session/state
                :expire-state
                :extract-sid
                :finalize-state
                :generate-sid)
  (:import-from :lack/middleware/session/store/memory
                :make-memory-store)
  (:import-from :lack/middleware/session/state/cookie
                :make-cookie-state)
  (:export :*lack-middleware-session*))
(in-package :lack/middleware/session)

(defparameter *lack-middleware-session*
  (lambda (app &key
                  (store (make-memory-store))
                  (state (make-cookie-state))
                  (keep-empty t))
    (lambda (env)
      (let* ((sid (extract-sid state env))
             (session (and sid
                           (fetch-session store sid)))
             (sid (or sid
                      (generate-sid state env)))
             (new-session-p (not session))
             (session (or session (make-hash-table :test 'equal))))
        (setf (getf env :lack.session) session)
        (setf (getf env :lack.session.options)
              (if new-session-p
                  (list :id sid :new-session t :change-id nil :expire nil)
                  (list :id sid :new-session nil :change-id nil :expire nil)))
        (let ((res (funcall app env))
              (process-session (lambda (result)
                                 (if (and (not keep-empty)
                                          new-session-p
                                          (zerop (hash-table-count session)))
                                     result
                                     (finalize store state env result)))))
          (typecase res
            (function (lambda (responder)
                        (funcall res (lambda (result)
                                       (funcall responder (funcall process-session result))))))
            (t (funcall process-session res)))))))
  "Middleware for session management")

(defun finalize (store state env res)
  (let* ((session (getf env :lack.session))
         (options (getf env :lack.session.options))
         (id (getf options :id))
         (new-id (if (getf options :change-id)
                     (generate-sid state env)
                     id)))
    (when session
      (apply #'commit store new-id session options))
    (if (getf options :expire)
        (expire-state state id res options)
        (finalize-state state new-id res options))))

(defun commit (store new-sid session &key id expire change-id &allow-other-keys)
  (cond
    (expire
     (remove-session store id))
    (change-id
     (remove-session store id)
     (store-session store new-sid session))
    (t
     (store-session store id session))))
