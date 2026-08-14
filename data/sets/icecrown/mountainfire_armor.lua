local card={api_version=1,id="ICC_062",name="Mountainfire Armor",text="<b>Deathrattle:</b> If it's your opponent's turn,\ngain 6 Armor.",set="ICECROWN",type="minion",class="warrior",rarity="rare",cost=3,attack=4,health=3,keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)if ctx:active_player()~=ctx:controller(self)then ctx:gain_armor(ctx:controller(self),6)end end
return card
