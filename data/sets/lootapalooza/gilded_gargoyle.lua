local card={api_version=1,id="LOOT_534",name="Gilded Gargoyle",text="<b>Deathrattle:</b> Add a Coin to your hand.",set="LOOTAPALOOZA",type="minion",class="priest",rarity="common",cost=3,attack=2,health=2,tags={"undead"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)cardlib.effects.give_card(ctx, ctx:controller(self),"GAME_005")end
return card
