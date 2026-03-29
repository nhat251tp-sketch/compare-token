# Compare

## Title
**Compare: So sanh phi chuyen tien SWIFT vs Stellar + mo phong remittance**

## Description
Compare la du an nham giup nguoi dung nhin thay su khac biet ve chi phi chuyen tien quoc te giua mo hinh truyen thong (SWIFT) va Stellar. Ngoai phep tinh phi, contract con cho phep tao remittance, thuc thi chuyen token va huy giao dich trong trang thai cho.

Muc dich cua du an:
- Truc quan hoa loi ich chi phi cua Stellar.
- Tao mot case study thuc te cho bai toan cross-border payment.
- Cung cap nen tang MVP de mo rong thanh san pham remittance cho nguoi dung pho thong.

Tai sao lam idea nay:
- Chuyen tien quoc te van la bai toan phi cao va cham.
- Stellar co uu the ro ve toc do va phi giao dich.
- Du an de trinh bay trong workshop/hackathon vi co tinh ung dung cao.

## Tinh nang
- So sanh phi SWIFT (uoc tinh 5%) va phi Stellar (`compare_fees`).
- Tao giao dich remittance moi (`create`).
- Thuc thi chuyen token tu nguoi gui sang nguoi nhan (`execute`).
- Huy giao dich khi chua thuc hien (`cancel`).
- Truy van thong tin remittance theo ID (`get`).
- Quan ly trang thai giao dich: `Pending`, `Completed`, `Cancelled`.
- Tich hop goi chuyen token cross-contract qua token client Soroban.

## Contract
- Contract ID: `CDMDBCPE5ISAGYS3E6FO3MMILASECIZGCHZNDTWN5N55IFS5GQQLS5MB`
- Explorer Link: https://stellar.expert/explorer/testnet/contract/CDMDBCPE5ISAGYS3E6FO3MMILASECIZGCHZNDTWN5N55IFS5GQQLS5MB?filter=history

Anh chup man hinh contract:
- Chua cap nhat (ban co the them anh vao README sau khi chup Stellar Expert).

## Future scopes
- Them dynamic fee model dua tren quoc gia/kenh nhan.
- Tich hop quote theo thoi gian thuc va ty gia on-chain/off-chain.
- Them lich su giao dich + dashboard phan tich tiet kiem phi cho nguoi dung.
- Mo rong thanh remittance app co frontend wallet-ready.

## Profile
- name: nhat nguyen
