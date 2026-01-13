' Licensed to the .NET Foundation under one or more agreements.
' The .NET Foundation licenses this file to you under the MIT license.
' See the LICENSE file in the project root for more information.

Imports System.Collections.Concurrent
Imports System.Collections.Immutable
Imports System.Threading
Imports Microsoft.CodeAnalysis.PooledObjects
Imports Microsoft.CodeAnalysis.VisualBasic
Imports Microsoft.CodeAnalysis.VisualBasic.Symbols
Imports Microsoft.CodeAnalysis.VisualBasic.Syntax

Namespace Microsoft.CodeAnalysis.Operations
    Partial Friend NotInheritable Class VisualBasicOperationFactory

        Private _lazyPlaceholderToParentMap As ConcurrentDictionary(Of BoundValuePlaceholderBase, BoundNode) = Nothing

        Private ReadOnly _semanticModel As SemanticModel

        Public Sub New(semanticModel As SemanticModel)
            _semanticModel = semanticModel
        End Sub

        ''' <summary>
        ''' Returns <code>Nothing</code> if parent is not known.
        ''' </summary>
        Private Function TryGetParent(placeholder As BoundValuePlaceholderBase) As BoundNode
            Dim knownParent As BoundNode = Nothing

            If _lazyPlaceholderToParentMap IsNot Nothing AndAlso
               _lazyPlaceholderToParentMap.TryGetValue(placeholder, knownParent) Then
                Return knownParent
            End If

            Return Nothing
        End Function

        Private Sub RecordParent(placeholderOpt As BoundValuePlaceholderBase, parent As BoundNode)
            Debug.Assert(parent IsNot Nothing)

            If placeholderOpt Is Nothing Then
                Return
            End If

            If _lazyPlaceholderToParentMap Is Nothing Then
                Interlocked.CompareExchange(_lazyPlaceholderToParentMap,
                                            New ConcurrentDictionary(Of BoundValuePlaceholderBase, BoundNode)(concurrencyLevel:=2, capacity:=10, comparer:=ReferenceEqualityComparer.Instance),
                                            Nothing)
            End If

            Dim knownParent = _lazyPlaceholderToParentMap.GetOrAdd(placeholderOpt, parent)
            Debug.Assert(knownParent Is parent)
        End Sub

        Public Function Create(boundNode As BoundNode) As IOperation
            If boundNode Is Nothing Then
                Return Nothing
            End If

            ' A BoundUserDefined conversion is always the operand of a BoundConversion, and is handled
            ' by the BoundConversion creation. We should never receive one in this top level create call.
            Debug.Assert(boundNode.Kind <> BoundKind.UserDefinedConversion)

            Select Case boundNode.Kind
                Case BoundKind.AssignmentOperator
                    Return CreateBoundAssignmentOperatorOperation(DirectCast(boundNode, BoundAssignmentOperator))
                Case BoundKind.MeReference
                    Return CreateBoundMeReferenceOperation(DirectCast(boundNode, BoundMeReference))
                Case BoundKind.MyBaseReference
                    Return CreateBoundMyBaseReferenceOperation(DirectCast(boundNode, BoundMyBaseReference))
                Case BoundKind.MyClassReference
                    Return CreateBoundMyClassReferenceOperation(DirectCast(boundNode, BoundMyClassReference))
                Case BoundKind.Literal
                    Return CreateBoundLiteralOperation(DirectCast(boundNode, BoundLiteral))
                Case BoundKind.AwaitOperator
                    Return CreateBoundAwaitOperatorOperation(DirectCast(boundNode, BoundAwaitOperator))
                Case BoundKind.NameOfOperator
                    Return CreateBoundNameOfOperatorOperation(DirectCast(boundNode, BoundNameOfOperator))
                Case BoundKind.Lambda
                    Return CreateBoundLambdaOperation(DirectCast(boundNode, BoundLambda))
                Case BoundKind.Call
                    Return CreateBoundCallOperation(DirectCast(boundNode, BoundCall))
                Case BoundKind.OmittedArgument
                    Return CreateBoundOmittedArgumentOperation(DirectCast(boundNode, BoundOmittedArgument))
                Case BoundKind.Parenthesized
                    Return CreateBoundParenthesizedOperation(DirectCast(boundNode, BoundParenthesized))
                Case BoundKind.ArrayAccess
                    Return CreateBoundArrayAccessOperation(DirectCast(boundNode, BoundArrayAccess))
                Case BoundKind.UnaryOperator
                    Return CreateBoundUnaryOperatorOperation(DirectCast(boundNode, BoundUnaryOperator))
                Case BoundKind.UserDefinedUnaryOperator
                    Return CreateBoundUserDefinedUnaryOperatorOperation(DirectCast(boundNode, BoundUserDefinedUnaryOperator))
                Case BoundKind.BinaryOperator
                    Return CreateBoundBinaryOperatorOperation(DirectCast(boundNode, BoundBinaryOperator))
                Case BoundKind.UserDefinedBinaryOperator
                    Return CreateBoundUserDefinedBinaryOperatorOperation(DirectCast(boundNode, BoundUserDefinedBinaryOperator))
                Case BoundKind.BinaryConditionalExpression
                    Return CreateBoundBinaryConditionalExpressionOperation(DirectCast(boundNode, BoundBinaryConditionalExpression))
                Case BoundKind.UserDefinedShortCircuitingOperator
                    Return CreateBoundUserDefinedShortCircuitingOperatorOperation(DirectCast(boundNode, BoundUserDefinedShortCircuitingOperator))
                Case BoundKind.BadExpression
                    Return CreateBoundBadExpressionOperation(DirectCast(boundNode, BoundBadExpression))
                Case BoundKind.TryCast
                    Return CreateBoundTryCastOperation(DirectCast(boundNode, BoundTryCast))
                Case BoundKind.DirectCast
                    Return CreateBoundDirectCastOperation(DirectCast(boundNode, BoundDirectCast))
                Case BoundKind.Conversion
                    Return CreateBoundConversionOperation(DirectCast(boundNode, BoundConversion))
                Case BoundKind.DelegateCreationExpression
                    Return CreateBoundDelegateCreationExpressionOperation(DirectCast(boundNode, BoundDelegateCreationExpression))
                Case BoundKind.TernaryConditionalExpression
                    Return CreateBoundTernaryConditionalExpressionOperation(DirectCast(boundNode, BoundTernaryConditionalExpression))
                Case BoundKind.TypeOf
                    Return CreateBoundTypeOfOperation(DirectCast(boundNode, BoundTypeOf))
                Case BoundKind.GetType
                    Return CreateBoundGetTypeOperation(DirectCast(boundNode, BoundGetType))
                Case Else
                    Return CreateInvalidOperation(boundNode)
            End Select
        End Function

    End Class
End Namespace
