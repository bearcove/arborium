/*
 * Copyright 2010-2020 JetBrains s.r.o. and Kotlin Programming Language contributors.
 * Use of this source code is governed by the Apache 2.0 license that can be found in the license/LICENSE.txt file.
 */

@file:kotlin.jvm.JvmMultifileClass
@file:kotlin.jvm.JvmName("SequencesKt")

package kotlin.sequences

import kotlin.internal.InlineOnly
import kotlin.random.Random

/**
 * Given an [iterator] function constructs a [Sequence] that returns values through the [Iterator]
 * provided by that function.
 * The values are evaluated lazily, and the sequence is potentially infinite.
 *
 * @sample samples.collections.Sequences.Building.sequenceFromIterator
 */
@kotlin.internal.InlineOnly
public inline fun <T> Sequence(crossinline iterator: () -> Iterator<T>): Sequence<T> = object : Sequence<T> {
    override fun iterator(): Iterator<T> = iterator()
}

/**
 * Creates a sequence that returns all elements from this iterator. The sequence is constrained to be iterated only once.
 *
 * @sample samples.collections.Sequences.Building.sequenceFromIterator
 */
public fun <T> Iterator<T>.asSequence(): Sequence<T> = Sequence { this }.constrainOnce()

/**
 * Creates a sequence that returns the specified values.
 *
 * @sample samples.collections.Sequences.Building.sequenceOfValues
 */
public fun <T> sequenceOf(vararg elements: T): Sequence<T> = elements.asSequence()

/**
 * Creates a [Sequence] that contains a single given element.
 *
 * @param element the single element to be contained in the resulting sequence.
 * @return a sequence containing only the specified [element].
 * @sample samples.collections.Sequences.Building.sequenceOfSingleValue
 */
@SinceKotlin("2.2")
public fun <T> sequenceOf(element: T): Sequence<T> = Sequence {
    object : Iterator<T> {
        private var _hasNext: Boolean = true

        override fun next(): T {
            if (!_hasNext) throw NoSuchElementException()
            _hasNext = false
            return element
        }

        override fun hasNext(): Boolean = _hasNext
    }
}

/**
 * Creates an empty [Sequence].
 *
 * @return an empty sequence.
 * @sample samples.collections.Sequences.Building.sequenceOfEmpty
 */
@SinceKotlin("2.2")
@InlineOnly
public inline fun <T> sequenceOf(): Sequence<T> = emptySequence()

/**
 * Returns an empty sequence.
 */
public fun <T> emptySequence(): Sequence<T> = EmptySequence

private object EmptySequence : Sequence<Nothing>, DropTakeSequence<Nothing> {
    override fun iterator(): Iterator<Nothing> = EmptyIterator
    override fun drop(n: Int) = EmptySequence
    override fun take(n: Int) = EmptySequence
}

/**
 * Returns this sequence if it's not `null` and the empty sequence otherwise.
 * @sample samples.collections.Sequences.Usage.sequenceOrEmpty
 */
@SinceKotlin("1.3")
@kotlin.internal.InlineOnly
public inline fun <T> Sequence<T>?.orEmpty(): Sequence<T> = this ?: emptySequence()


/**
 * Returns a sequence that iterates through the elements either of this sequence
 * or, if this sequence turns out to be empty, of the sequence returned by [defaultValue] function.
 *
 * @sample samples.collections.Sequences.Usage.sequenceIfEmpty
 */
@SinceKotlin("1.3")
public fun <T> Sequence<T>.ifEmpty(defaultValue: () -> Sequence<T>): Sequence<T> = sequence {
    val iterator = this@ifEmpty.iterator()
    if (iterator.hasNext()) {
        yieldAll(iterator)
    } else {
        yieldAll(defaultValue())
    }
}

/**
 * Returns a sequence of all elements from all sequences in this sequence.
 *
 * The operation is _intermediate_ and _stateless_.
 *
 * @sample samples.collections.Sequences.Transformations.flattenSequenceOfSequences
 */
public fun <T> Sequence<Sequence<T>>.flatten(): Sequence<T> = flatten { it.iterator() }

/**
 * Returns a sequence of all elements from all iterables in this sequence.
 *
 * The operation is _intermediate_ and _stateless_.
 *
 * @sample samples.collections.Sequences.Transformations.flattenSequenceOfLists
 */
@kotlin.jvm.JvmName("flattenSequenceOfIterable")
public fun <T> Sequence<Iterable<T>>.flatten(): Sequence<T> = flatten { it.iterator() }
