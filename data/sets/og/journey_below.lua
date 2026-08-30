local function deathrattle(d)for _,k in ipairs(d.keywords or {})do if k=="deathrattle" then return true end end return false end
local card={api_version=1,id="OG_072",name="Journey Below",text="<b>Discover</b> a <b>Deathrattle</b> card.",set="OG",type="spell",class="rogue",rarity="rare",cost=1}
function card.on_play(ctx,self)local p=ctx:controller(self);local class=ctx:player(p).class;local pool={};for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if deathrattle(d) and (d.class=="neutral" or d.class==class)then pool[#pool+1]=id end end;if #pool>0 then ctx:discover_cards(p,"Choose a Deathrattle card",pool,3,"receive_card")end end
function card.receive_card(ctx,self,id)cardlib.effects.give_card(ctx, ctx:controller(self),id)end
return card
