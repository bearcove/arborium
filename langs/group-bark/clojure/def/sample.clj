(defn fibonacci [n]
  "Generate Fibonacci sequence"
  (loop [a 0 b 1 result []]
    (if (>= (count result) n)
      result
      (recur b (+ a b) (conj result a)))))

(println (fibonacci 10))
