#lang racket/base

(provide first second third fourth fifth sixth seventh eighth ninth tenth
         eleventh twelfth thirteenth fourteenth fifteenth
         last-pair last rest
         cons? empty empty?
         make-list list-update list-set
         index-of index-where indexes-of indexes-where
         drop take split-at takef dropf splitf-at drop-right take-right
         split-at-right takef-right dropf-right splitf-at-right
         list-prefix? split-common-prefix take-common-prefix drop-common-prefix
         append* flatten add-between remove-duplicates check-duplicates
         filter-map count partition
         range inclusive-range append-map filter-not shuffle combinations
         in-combinations permutations in-permutations
         argmin argmax group-by cartesian-product remf remf*)

(require (for-syntax racket/base)
         racket/private/list-predicates)

(define (first x)
  (if (and (pair? x) (list? x))
    (car x)
    (raise-argument-error 'first
      "(and/c list? (not/c empty?))" x)))

(define-syntax define-lgetter
  (syntax-rules ()
    [(_ name npos)
     (define (name l0)
       (if (list? l0)
         (let loop ([l l0] [pos npos])
           (if (pair? l)
             (if (eq? pos 1) (car l) (loop (cdr l) (sub1 pos)))
             (raise-arguments-error 'name
                "list contains too few elements"
                "list" l0)))
         (raise-argument-error 'name "list?" l0)))]))

(define-lgetter second      2)
(define-lgetter third       3)
(define-lgetter fourth      4)
(define-lgetter fifth       5)
(define-lgetter sixth       6)
(define-lgetter seventh     7)
(define-lgetter eighth      8)
(define-lgetter ninth       9)
(define-lgetter tenth      10)
(define-lgetter eleventh   11)
(define-lgetter twelfth    12)
(define-lgetter thirteenth 13)
(define-lgetter fourteenth 14)
(define-lgetter fifteenth  15)

(define (last-pair l)
  (if (pair? l)
    (let loop ([l l] [x (cdr l)])
      (if (pair? x)
        (loop x (cdr x))
        l))
    (raise-argument-error 'last-pair "pair?" l)))

(define (last l)
  (if (and (pair? l) (list? l))
    (let loop ([l l] [x (cdr l)])
      (if (pair? x)
        (loop x (cdr x))
        (car l)))
    (raise-argument-error 'last
      "(and/c list? (not/c empty?))" l)))

(define (rest l)
  (if (and (pair? l) (list? l))
    (cdr l)
    (raise-argument-error 'rest
      "(and/c list? (not/c empty?))" l)))

(define empty '())

(define (make-list n x)
  (unless (exact-nonnegative-integer? n)
    (raise-argument-error 'make-list
      "exact-nonnegative-integer?" 0 n x))
  (let loop ([n n] [r '()])
    (if (zero? n) r (loop (sub1 n) (cons x r)))))

(define (list-update l i f)
  (unless (list? l)
    (raise-argument-error 'list-update "list?" 0 l i f))
  (unless (exact-nonnegative-integer? i)
    (raise-argument-error 'list-update
      "exact-nonnegative-integer?" 1 l i f))
  (unless (and (procedure? f)
               (procedure-arity-includes? f 1))
    (raise-argument-error 'list-update
      "(-> any/c any/c)" 2 l i f))
  (cond
   [(zero? i) (cons (f (car l)) (cdr l))]
   [else (cons (car l) (list-update (cdr l) (sub1 i) f))]))

(define (list-set l k v)
  (unless (list? l)
    (raise-argument-error 'list-set "list?" 0 l k v))
  (unless (exact-nonnegative-integer? k)
    (raise-argument-error 'list-set
      "exact-nonnegative-integer?" 1 l k v))
  (list-update l k (lambda (_) v)))
