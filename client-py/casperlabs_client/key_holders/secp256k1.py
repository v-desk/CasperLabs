from typing import Union
from pathlib import Path

import ecdsa
from casperlabs_client.consts import SECP256K1_KEY_ALGORITHM
from casperlabs_client.io import read_binary_file
from .key_holder import KeyHolder


class SECP256K1Key(KeyHolder):
    """
    Class for loading, generating and handling public/private keys using secp256k1 algorithm

    Note: Many ecdsa methods are from/to_string. This is hold over from Python2 and work as bytes in Python3
    """

    CURVE = ecdsa.SECP256k1

    def __init__(
        self,
        private_key_pem: bytes = None,
        private_key=None,
        public_key_pem: bytes = None,
        public_key=None,
    ):
        super().__init__(
            private_key_pem,
            private_key,
            public_key_pem,
            public_key,
            SECP256K1_KEY_ALGORITHM,
        )

    @property
    def private_key_pem(self):
        """ Returns or generates private key pem data from other internal fields """
        if self._private_key_pem is None:
            if self._private_key is None:
                raise ValueError("Must have either _private_key or _private_key_pem.")
            private_key_object = ecdsa.SigningKey.from_string(
                self._private_key, curve=self.CURVE
            )
            self._private_key_pem = private_key_object.to_pem()
        return self._private_key_pem

    @property
    def private_key(self):
        """ Returns or generates private key bytes from other internal fields """
        if self._private_key is None:
            if self._private_key_pem is None:
                raise ValueError("Must have either _private_key or _private_key_pem.")
            private_key_object = ecdsa.SigningKey.from_pem(self._private_key_pem)
            self._private_key = private_key_object.to_string()
        return self._private_key

    @property
    def public_key_pem(self):
        """ Returns or generates public key pem data from other internal fields """
        if self._public_key_pem is None:
            public_key_object = ecdsa.VerifyingKey.from_string(
                self.public_key, curve=self.CURVE
            )
            self._public_key_pem = public_key_object.to_pem()
        return self._public_key_pem

    @property
    def public_key(self):
        """ Returns or generates public key bytes from other internal fields """
        if self._public_key is None:
            if self._public_key_pem:
                public_key_object = ecdsa.VerifyingKey.from_pem(self._public_key_pem)
                self._public_key = public_key_object.to_string()
            elif self.private_key:
                private_key_object = ecdsa.SigningKey.from_string(
                    self.private_key, curve=self.CURVE
                )
                self._public_key = private_key_object.verifying_key.to_string()
            else:
                raise ValueError("No values given to derive public key")
        return self._public_key

    def sign(self, data: bytes) -> bytes:
        """ Return signature of data given """
        private_key_object = ecdsa.SigningKey.from_string(
            self.private_key, curve=self.CURVE
        )
        return private_key_object.sign(data)

    @staticmethod
    def generate():
        """
        Generates a new key pair and returns as SEPC256K1Key object.

        :returns SECP256K1Key object
        """
        private_key_object = ecdsa.SigningKey.generate(curve=SECP256K1Key.CURVE)
        private_key = private_key_object.to_string()
        return SECP256K1Key(private_key=private_key)

    @staticmethod
    def from_private_key_path(private_key_pem_path: Union[str, Path]) -> "KeyHolder":
        """ Creates SECP256K1Key object from private key file in pem format """
        private_key_pem = read_binary_file(private_key_pem_path)
        return SECP256K1Key(private_key_pem=private_key_pem)

    @staticmethod
    def from_public_key_path(public_key_pem_path: Union[str, Path]) -> "KeyHolder":
        """
        Creates SECP256K1Key object from public key file in pem format.

        Note: Functionality requiring Private Key will not be possible.  Use only if no private key pem is available.
        """
        public_key_pem = read_binary_file(public_key_pem_path)
        return SECP256K1Key(public_key_pem=public_key_pem)

    @staticmethod
    def from_private_key(private_key: bytes) -> "KeyHolder":
        """ Creates SECP256K1Key object from private key in bytes """
        return SECP256K1Key(private_key=private_key)
