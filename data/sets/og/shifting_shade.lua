local card={api_version=1,id="OG_335",name="Shifting Shade",text="[x]<b>Deathrattle:</b> Copy a card\nfrom your opponent's deck\n and add it to your hand.",set="OG",type="minion",class="priest",rarity="rare",cost=4,attack=4,health=3,tags={"undead"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)local deck=ctx:deck(ctx:opponent(ctx:controller(self)));if #deck>0 then ctx:random_value(deck,"copy_card")end end
function card.copy_card(ctx,self,target)ctx:give_copy(ctx:controller(self),target)end
return card
