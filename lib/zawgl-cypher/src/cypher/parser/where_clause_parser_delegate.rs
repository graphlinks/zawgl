// MIT License
//
// Copyright (c) 2022 Alexandre RICCIARDI
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::*;
use super::error::*;

use zawgl_cypher_query_model::ast::{AstTagNode, AstTag, AstTokenNode, Ast, AstVisitorResult, AstVisitor};
use zawgl_cypher_query_model::token::{TokenType, Token};
use super::common_parser_delegate::*;

pub fn parse_where_clause(parser: &mut Parser, parent_node: &mut Box<AstTagNode>) -> ParserResult<()> {
    if parser.check(TokenType::Where) {
        parser.require(TokenType::Where)?;
        let mut where_node = make_ast_tag(AstTag::Where);
        where_node.append(parse_boolean_expression(parser)?);
        parent_node.append(where_node);
    }
    Ok(())
}

fn parse_boolean_operator(parser: &mut Parser, prev_expr: Box<AstTagNode>) -> ParserResult<Box<AstTagNode>> {
    if parser.check(TokenType::And) {
        parser.advance();
        let mut operator = make_ast_tag(AstTag::AndOperator);
        let mut expr = parse_boolean_expression(parser)?;
        if expr.ast_tag == Some(AstTag::OrOperator) {
            operator.append(prev_expr);
            operator.append(expr.childs.remove(0));
            expr.append(operator);
            Ok(expr)
        } else {
            operator.append(prev_expr);
            operator.append(expr);
            Ok(operator)
        }
        
    } else if parser.check(TokenType::Or) {
        parser.advance();
        let mut operator = make_ast_tag(AstTag::OrOperator);
        let expr = parse_boolean_expression(parser)?;
        operator.append(prev_expr);
        operator.append(expr);
        Ok(operator)
    } else {
        Ok(prev_expr)
    }
}

fn parse_boolean_expression_terminal(parser: &mut Parser, parent_node: &mut Box<AstTagNode>) -> ParserResult<()> {
    match parser.get_current_token_type() {
        TokenType::Integer | TokenType::Float | TokenType::True | TokenType::False | TokenType::StringType | TokenType::Parameter => {
            parser.advance();
            parent_node.append(make_ast_token(parser));
            Ok(())
        },
        TokenType::Identifier => {
            parser.advance();
            if parser.check(TokenType::OpenParenthesis) {
                let func = parse_function_definition(parser)?;
                parent_node.append(func);
            } else if parser.check(TokenType::Dot) {
                let mut item_prop = make_ast_tag(AstTag::ItemPropertyIdentifier);
                item_prop.append(make_ast_token(parser));
                parser.advance();
                if parser.check(TokenType::Identifier) {
                    parser.advance();
                    item_prop.append(make_ast_token(parser));
                } else {
                    return Err(ParserError::SyntaxError(parser.index))
                }
            } else {
                return Err(ParserError::SyntaxError(parser.index))
            }
            Ok(())
        },
        _ => {
            Err(ParserError::SyntaxError(parser.index))
        }
    }
}

fn parse_boolean_expression(parser: &mut Parser) -> ParserResult<Box<AstTagNode>> {
    match parser.get_current_token_type() {
        TokenType::Integer => {
            parser.advance();
            parser.require(TokenType::Equals)?;
            let mut eqop = make_ast_tag(AstTag::EqualityOperator);
            parse_boolean_expression_terminal(parser, &mut eqop)?;
            parse_boolean_operator(parser, eqop)
        },
        TokenType::Float => {
            parser.advance();
            parser.require(TokenType::Equals)?;
            let mut eqop = make_ast_tag(AstTag::EqualityOperator);
            parse_boolean_expression_terminal(parser, &mut eqop)?;
            parse_boolean_operator(parser, eqop)
        },
        TokenType::Parameter => {
            parser.advance();
            parser.require(TokenType::Equals)?;
            let mut eqop = make_ast_tag(AstTag::EqualityOperator);
            parse_boolean_expression_terminal(parser, &mut eqop)?;
            parse_boolean_operator(parser, eqop)
        }
        TokenType::True | TokenType::False => {
            parser.advance();
            parser.require(TokenType::Equals)?;
            let mut eqop = make_ast_tag(AstTag::EqualityOperator);
            parse_boolean_expression_terminal(parser, &mut eqop)?;
            parse_boolean_operator(parser, eqop)
        },
        TokenType::Identifier => {
            parser.advance();
            
            if parser.check(TokenType::OpenParenthesis) {
                let func = parse_function_definition(parser)?;
                if parser.check(TokenType::Equals) {
                    parser.advance();
                    let mut eqop = make_ast_tag(AstTag::EqualityOperator);
                    eqop.append(func);
                    parse_boolean_expression_terminal(parser, &mut eqop)?;
                    return parse_boolean_operator(parser, eqop)
                } else {
                    return parse_boolean_operator(parser, func)
                }
            } else if parser.check(TokenType::Dot) {
                let mut item_prop = make_ast_tag(AstTag::ItemPropertyIdentifier);
                item_prop.append(make_ast_token(parser));
                parser.advance();
                if parser.check(TokenType::Identifier) {
                    parser.advance();
                    item_prop.append(make_ast_token(parser));
                    parser.require(TokenType::Equals)?;
                    let mut eqop = make_ast_tag(AstTag::EqualityOperator);
                    eqop.append(item_prop);
                    parse_boolean_expression_terminal(parser, &mut eqop)?;
                    return parse_boolean_operator(parser, eqop)
                } else {
                    return Err(ParserError::SyntaxError(parser.index))
                }
            } else {
                return Err(ParserError::SyntaxError(parser.index))
            }
        },
        TokenType::OpenParenthesis => {
            parser.advance();
            let expr = parse_boolean_expression(parser)?;
            parser.require(TokenType::CloseParenthesis)?;
            return parse_boolean_operator(parser, expr)
        },
        _ => {
            Err(ParserError::SyntaxError(parser.index))
        }
    }
}
