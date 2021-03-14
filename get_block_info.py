import requests
import json
import time
import datetime
from datetime import timezone, timedelta
import hashlib
from binascii import unhexlify, hexlify
import array

def getPrevBlock(block):
  r = requests.get(block["prev_block_url"])
  return json.loads(r.text)

def flipBytes(hexStr):
  arr = bytearray(unhexlify(hexStr))
  arr.reverse()
  return hexlify(arr).decode("utf-8")


def createBlockHash(block):
  # This is signed
  ver = int(block["ver"])
  prev = block["prev_block"]
  root = block["mrkl_root"]
  # this is a string
  ts = block["time"]
  # These are unsigned ints
  bits = int(block["bits"])
  nonce = int(block["nonce"])
  
  ver = hexlify(ver.to_bytes(4,byteorder='little', signed=True)).decode("utf-8")
  # flip the bytes for the hashes to little endian
  prev = flipBytes(prev)
  root = flipBytes(root)

  epochTime = datetime.datetime.strptime(ts,"%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
  ts = hexlify(int(epochTime.timestamp()).to_bytes(4,byteorder='little',signed=True)).decode("utf-8")


  bits = hexlify(bits.to_bytes(4,byteorder='little')).decode("utf-8")
  nonce = hexlify(nonce.to_bytes(4,byteorder='little')).decode("utf-8")

  print([ver, prev, root, ts, bits, nonce])
  return double_hash(ver + prev + root + ts + bits + nonce)
  

def double_hash(hex_in):
  header_bin = unhexlify(hex_in)
  hash = hashlib.sha256(hashlib.sha256(header_bin).digest()).digest()
  return hexlify(hash[::-1]).decode("utf-8")
  
r = requests.get("https://api.blockcypher.com/v1/btc/main")
t = json.loads(r.text)
r = requests.get(t["latest_url"])
currBlock = json.loads(r.text)


for i in range(10):
  print(currBlock["hash"])
  print(createBlockHash(currBlock))
  currBlock = getPrevBlock(currBlock)
  print("\n")

'''
block = json.loads(requests.get("https://api.blockcypher.com/v1/btc/main/blocks/00000000000000001e8d6829a8a21adc5d38d0a473b144b6765798e61f98bd1d").text)

print(createBlockHash(block))
'''

