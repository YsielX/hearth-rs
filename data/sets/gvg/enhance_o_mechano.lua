local card = {
    api_version = 1, id = "GVG_107", name = "Enhance-o Mechano",
    text = "<b>Battlecry:</b> Give your other minions <b>Windfury</b>, <b>Taunt</b>, or <b>Divine Shield</b>\n<i>(at random)</i>.",
    set = "GVG", type = "minion", rarity = "epic", cost = 4, attack = 3, health = 2,
    tags = { "mech" }, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    for _, target in ipairs(ctx:friendly_minions(self)) do
        if target ~= self then
            ctx:random_value({
                { target = target, keyword = "windfury" },
                { target = target, keyword = "taunt" },
                { target = target, keyword = "divine_shield" },
            }, "enhance")
        end
    end
end
function card.enhance(ctx, self, choice) ctx:grant_keyword(choice.target, choice.keyword) end
return card
