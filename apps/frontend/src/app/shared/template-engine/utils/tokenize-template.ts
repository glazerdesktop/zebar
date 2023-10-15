import { createStringScanner } from '~/shared/utils';
import { Token, TokenType } from '../types';
import { TemplateError } from './template-error';

export enum TokenizeStateType {
  DEFAULT,
  IN_STATEMENT_ARGS,
  IN_STATEMENT_BLOCK,
  IN_INTERPOLATION,
  IN_EXPRESSION,
}

export interface InExpressionState {
  type: TokenizeStateType.IN_EXPRESSION;
  token?: Token;
  closeRegex: RegExp;
  ignoreSymbol: string | null;
}

export type TokenizeState = { type: TokenizeStateType } | InExpressionState;

export function tokenizeTemplate(template: string): Token[] {
  // Stack of tokenize states. Last element represents current state.
  const stateStack: TokenizeState[] = [{ type: TokenizeStateType.DEFAULT }];

  // Tokens within input template.
  const tokens: Token[] = [];

  // String scanner for advancing through input template.
  const scanner = createStringScanner(template);

  function pushToken(typeOrToken: TokenType | Token) {
    const token =
      typeof typeOrToken === 'object'
        ? typeOrToken
        : { type: typeOrToken, ...scanner.latestMatch! };

    console.log('token', token, tokens, scanner);

    if (!token.substring) {
      throw new TemplateError('Cannot push an empty token.', scanner.cursor);
    }

    tokens.push(token);
  }

  // Push a tokenize state.
  function pushState(typeOrState: TokenizeStateType | TokenizeState) {
    const state =
      typeof typeOrState === 'object' ? typeOrState : { type: typeOrState };

    stateStack.push(state);
  }

  function updateLatestState(state: TokenizeState) {
    stateStack[stateStack.length - 1] = state;
  }

  // Get current tokenize state.
  function getState() {
    return stateStack[stateStack.length - 1];
  }

  while (!scanner.isEmpty) {
    switch (getState().type) {
      case TokenizeStateType.DEFAULT:
        tokenizeDefault();
        break;
      case TokenizeStateType.IN_STATEMENT_ARGS:
        tokenizeStatementArgs();
        break;
      case TokenizeStateType.IN_STATEMENT_BLOCK:
        tokenizeStatementBlock();
        break;
      case TokenizeStateType.IN_INTERPOLATION:
        tokenizeInterpolation();
        break;
      case TokenizeStateType.IN_EXPRESSION:
        tokenizeExpression();
        break;
    }
  }

  function tokenizeDefault() {
    if (scanner.scan(/@if/)) {
      pushToken(TokenType.IF_STATEMENT);
      pushState(TokenizeStateType.IN_STATEMENT_ARGS);
    } else if (scanner.scan(/@else\s+if/)) {
      pushToken(TokenType.ELSE_IF_STATEMENT);
      pushState(TokenizeStateType.IN_STATEMENT_ARGS);
    } else if (scanner.scan(/@else/)) {
      pushToken(TokenType.ELSE_STATEMENT);
      pushState(TokenizeStateType.IN_STATEMENT_ARGS);
    } else if (scanner.scan(/@for/)) {
      pushToken(TokenType.FOR_STATEMENT);
      pushState(TokenizeStateType.IN_STATEMENT_ARGS);
    } else if (scanner.scan(/@switch/)) {
      pushToken(TokenType.SWITCH_STATEMENT);
      pushState(TokenizeStateType.IN_STATEMENT_ARGS);
    } else if (scanner.scan(/@case/)) {
      pushToken(TokenType.SWITCH_CASE_STATEMENT);
      pushState(TokenizeStateType.IN_STATEMENT_ARGS);
    } else if (scanner.scan(/@default/)) {
      pushToken(TokenType.SWITCH_DEFAULT_STATEMENT);
      pushState(TokenizeStateType.IN_STATEMENT_ARGS);
    } else if (scanner.scan(/{{/)) {
      pushToken(TokenType.OPEN_INTERPOLATION);
      pushState(TokenizeStateType.IN_INTERPOLATION);
    } else if (scanner.scanUntil(/.*?(?={{|@|})/)) {
      // Search until a close block, the start of a statement, or the start of
      // an interpolation tag.
      pushToken(TokenType.TEXT);
    } else {
      throw new TemplateError('No valid tokens found.', scanner.cursor);
    }
  }

  function tokenizeStatementArgs() {
    if (scanner.scan(/\)?\s+/)) {
      // Ignore whitespace within args, and closing parenthesis after
      // statement args.
    } else if (scanner.scan(/\(/)) {
      pushState({
        type: TokenizeStateType.IN_EXPRESSION,
        closeRegex: /.*?(?=\))/,
        ignoreSymbol: null,
      });
    } else if (scanner.scan(/{/)) {
      pushToken(TokenType.OPEN_BLOCK);
      stateStack.pop();
      pushState(TokenizeStateType.IN_STATEMENT_BLOCK);
    } else {
      throw new TemplateError('Missing closing {.', scanner.cursor);
    }
  }

  function tokenizeStatementBlock() {
    if (scanner.scan(/}/)) {
      pushToken(TokenType.CLOSE_BLOCK);
      stateStack.pop();
    } else {
      tokenizeDefault();
    }
  }

  function tokenizeInterpolation() {
    if (scanner.scan(/\s+/)) {
      // Ignore whitespace within interpolation tag.
    } else if (scanner.scan(/}}/)) {
      pushToken(TokenType.CLOSE_INTERPOLATION);
      stateStack.pop();
    } else if (scanner.scan(/.*?/)) {
      pushState({
        type: TokenizeStateType.IN_EXPRESSION,
        closeRegex: /.*?(?=\s*}})/,
        ignoreSymbol: null,
      });
    } else {
      throw new TemplateError('Missing closing }}.', scanner.cursor);
    }
  }

  function tokenizeExpression() {
    const state = getState() as InExpressionState;
    const { closeRegex, ignoreSymbol, token } = state;

    if (scanner.scan(/\s+/)) {
      // Ignore whitespace within expression.
      // } else if (scanner.scan(new RegExp(`.*?(?='|\`|"|${closeSymbol})/)`))) {
    } else if (scanner.scan(/.*?('|`|\(|")/)) {
      // Match expression until the close symbol or a string character. Closing
      // symbol should be ignored if wrapped within a string.
      const { startIndex, endIndex, substring } = scanner.latestMatch!;

      console.log('>', template, scanner.latestMatch);

      // Update expression token.
      state.token = {
        type: TokenType.EXPRESSION,
        startIndex: token?.startIndex ?? startIndex,
        endIndex,
        substring: (token?.substring ?? '') + substring,
      };

      const lookaheadChar = substring[substring.length - 1];

      if (
        lookaheadChar !== "'" &&
        lookaheadChar !== '`' &&
        lookaheadChar !== '"'
      ) {
        state.ignoreSymbol = null;
        return;
      }

      // Clear ignore symbol.
      if (ignoreSymbol && ignoreSymbol === lookaheadChar) {
        state.ignoreSymbol = null;
      }

      // Set ignore symbol to the current string character.
      if (!ignoreSymbol) {
        state.ignoreSymbol = lookaheadChar;
      }
    } else if (!ignoreSymbol && scanner.scan(closeRegex)) {
      console.log('aa', template, tokens, state, scanner);

      // if (!token) {
      //   throw new TemplateError('Missing expression.', scanner.cursor);
      // }

      const { startIndex, endIndex, substring } = scanner.latestMatch!;

      pushToken({
        type: TokenType.EXPRESSION,
        startIndex: token?.startIndex ?? startIndex,
        endIndex,
        substring: (token?.substring ?? '') + substring,
      });

      stateStack.pop();
    } else {
      console.log('err', template, state, scanner);

      throw new TemplateError('Missing close symbol.', scanner.cursor);
    }
  }

  return tokens;
}
