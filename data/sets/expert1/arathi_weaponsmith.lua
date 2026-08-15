local card={api_version=1,id="EX1_398",name="Arathi Weaponsmith",text="<b>Battlecry:</b> Equip a 2/2 weapon.",set="EXPERT1",type="minion",class="warrior",rarity="common",cost=4,attack=3,health=3,keywords={"battlecry"},on_battlecry=function(ctx,self)ctx:equip_weapon(ctx:controller(self),"EX1_398t")end}
card.tokens={{id="EX1_398t",name="Battle Axe",text="",set="EXPERT1",type="weapon",class="warrior",collectible=false,cost=1,attack=2,health=2}}
return card
