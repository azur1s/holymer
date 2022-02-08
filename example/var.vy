(fun return_true true)

(fun main (do
    (def name "John")
    (if (return_true)
        (print name)
        (print "no"))))