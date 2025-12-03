# This file is a part of Julia. License is MIT: https://julialang.org/license

## array.jl: Dense arrays

"""
    DimensionMismatch([msg])

The objects called do not have matching dimensionality. Optional argument `msg` is a
descriptive error string.
"""
struct DimensionMismatch <: Exception
    msg::AbstractString
end
DimensionMismatch() = DimensionMismatch("")

## Type aliases for convenience ##
"""
    AbstractVector{T}

Supertype for one-dimensional arrays (or array-like types) with
elements of type `T`. Alias for [`AbstractArray{T,1}`](@ref).
"""
const AbstractVector{T} = AbstractArray{T,1}

"""
    AbstractMatrix{T}

Supertype for two-dimensional arrays (or array-like types) with
elements of type `T`. Alias for [`AbstractArray{T,2}`](@ref).
"""
const AbstractMatrix{T} = AbstractArray{T,2}

"""
    AbstractVecOrMat{T}

Union type of [`AbstractVector{T}`](@ref) and [`AbstractMatrix{T}`](@ref).
"""
const AbstractVecOrMat{T} = Union{AbstractVector{T}, AbstractMatrix{T}}
const RangeIndex = Union{<:BitInteger, AbstractRange{<:BitInteger}}
const DimOrInd = Union{Integer, AbstractUnitRange}
const IntOrInd = Union{Int, AbstractUnitRange}
const DimsOrInds{N} = NTuple{N,DimOrInd}
const NeedsShaping = Union{Tuple{Integer,Vararg{Integer}}, Tuple{OneTo,Vararg{OneTo}}}

"""
    Array{T,N} <: AbstractArray{T,N}

`N`-dimensional dense array with elements of type `T`.
"""
Array

"""
    Vector{T} <: AbstractVector{T}

One-dimensional dense array with elements of type `T`, often used to represent
a mathematical vector. Alias for [`Array{T,1}`](@ref).

See also [`empty`](@ref), [`similar`](@ref) and [`zero`](@ref) for creating vectors.
"""
const Vector{T} = Array{T,1}

"""
    Matrix{T} <: AbstractMatrix{T}

Two-dimensional dense array with elements of type `T`, often used to represent
a mathematical matrix. Alias for [`Array{T,2}`](@ref).

See also [`fill`](@ref), [`zeros`](@ref), [`undef`](@ref) and [`similar`](@ref)
for creating matrices.
"""
const Matrix{T} = Array{T,2}

"""
    VecOrMat{T}

Union type of [`Vector{T}`](@ref) and [`Matrix{T}`](@ref) which allows functions to accept either a Matrix or a Vector.

# Examples
```jldoctest
julia> Vector{Float64} <: VecOrMat{Float64}
true

julia> Matrix{Float64} <: VecOrMat{Float64}
true

julia> Array{Float64, 3} <: VecOrMat{Float64}
false
```
"""
const VecOrMat{T} = Union{Vector{T}, Matrix{T}}

"""
    DenseArray{T, N} <: AbstractArray{T,N}

`N`-dimensional dense array with elements of type `T`.
The elements of a dense array are stored contiguously in memory.
"""
DenseArray

"""
    DenseVector{T}

One-dimensional [`DenseArray`](@ref) with elements of type `T`. Alias for `DenseArray{T,1}`.
"""
const DenseVector{T} = DenseArray{T,1}

"""
    DenseMatrix{T}

Two-dimensional [`DenseArray`](@ref) with elements of type `T`. Alias for `DenseArray{T,2}`.
"""
const DenseMatrix{T} = DenseArray{T,2}

"""
    DenseVecOrMat{T}

Union type of [`DenseVector{T}`](@ref) and [`DenseMatrix{T}`](@ref).
"""
const DenseVecOrMat{T} = Union{DenseVector{T}, DenseMatrix{T}}

