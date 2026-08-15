local card={api_version=1,id="EX1_383",name="Tirion Fordring",text="<b><b>Divine Shield</b>,</b> <b>Taunt</b> <b>Deathrattle:</b> Equip a 5/3 Ashbringer.",set="EXPERT1",type="minion",class="paladin",rarity="legendary",cost=8,attack=8,health=8,keywords={"divine_shield","taunt","deathrattle"},on_deathrattle=function(ctx,self)ctx:equip_weapon(ctx:controller(self),"EX1_383t")end}
card.tokens={{id="EX1_383t",name="Ashbringer",text="",set="EXPERT1",type="weapon",class="paladin",collectible=false,cost=5,attack=5,health=3}}
return card
