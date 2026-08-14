local card={api_version=1,id="LOOT_370",name="Gather Your Party",text="<b>Recruit</b> a minion.",set="LOOTAPALOOZA",type="spell",class="warrior",rarity="rare",cost=6}
function card.on_play(ctx,self)local c={}for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="minion"then c[#c+1]=e end end;if #c>0 then ctx:random_value(c,"recruit_selected")end end
function card.recruit_selected(ctx,self,e)ctx:recruit(ctx:controller(self),e)end
return card
