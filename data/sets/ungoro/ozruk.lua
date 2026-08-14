local card={api_version=1,id="UNG_907",name="Ozruk",text="[x]<b>Taunt</b>\n<b>Battlecry:</b> Gain +5 Health\nfor each Elemental you\nplayed last turn.",set="UNGORO",type="minion",class="neutral",rarity="legendary",cost=8,attack=8,health=8,tags={"elemental"},keywords={"taunt","battlecry"}}
function card.on_battlecry(ctx,self) local n=0 for _,id in ipairs(ctx:cards_played_last_turn(ctx:controller(self))) do for _,t in ipairs(ctx:card_definition(id).tags or {}) do if t=="elemental" or t=="all" then n=n+1;break end end end if n>0 then ctx:buff(self,0,5*n) end end
return card
