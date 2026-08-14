local cards={"ICC_314t1","ICC_314t2","ICC_314t3","ICC_314t4","ICC_314t5","ICC_314t6","ICC_314t7","ICC_314t8"}
local card={api_version=1,id="ICC_854",name="Arfus",text="[x]<b>Deathrattle:</b> Add a random\n<b>Lich King</b> card to your hand.",set="ICECROWN",type="minion",rarity="legendary",cost=4,attack=2,health=2,tags={"undead","beast"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)ctx:random_value(cards,"receive_death_knight_card")end
function card.receive_death_knight_card(ctx,self,id)ctx:give_card(ctx:controller(self),id)end
return card
