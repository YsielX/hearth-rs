local card={api_version=1,id="ICC_853",name="Prince Valanar",text="<b>Battlecry:</b> If your deck has no 4-Cost cards, gain <b>Lifesteal</b> and <b>Taunt</b>.",set="ICECROWN",type="minion",rarity="legendary",cost=4,attack=4,health=4,tags={"undead"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).cost==4 then return end end;cardlib.effects.grant_keyword(ctx, self,"lifesteal");cardlib.effects.grant_keyword(ctx, self,"taunt")end
return card
