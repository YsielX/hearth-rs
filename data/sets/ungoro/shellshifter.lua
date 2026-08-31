local card = { api_version = 1, id = "UNG_101", name = "Shellshifter",
    text = "[x]<b>Choose One - </b>Transform\ninto a 5/3 with <b>Stealth</b>;\nor a 3/5 with <b>Taunt</b>.",
    set = "UNGORO", type = "minion", class = "druid", rarity = "rare",
    cost = 4, attack = 3, health = 3, keywords = { "choose_one" } }
function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "UNG_101a", label = "5/3 with Stealth" }, { card_id = "UNG_101b", label = "3/5 with Taunt" },
    }, "chosen")
end
function card.chosen(ctx, self, choice) cardlib.effects.transform(ctx, self, choice == "UNG_101a" and "UNG_101t" or "UNG_101t2") end
function card.on_choose_multiple(ctx, self) cardlib.effects.transform(ctx, self, "UNG_101t3") end
card.tokens = {
    { id = "UNG_101a", name = "Raptor Form", text = "<b>Stealth</b>", set = "UNGORO", type = "minion", class = "druid", rarity = "rare", cost = 4, attack = 5, health = 3, tags = { "beast" }, keywords = { "stealth" } },
    { id = "UNG_101b", name = "Direhorn Form", text = "<b>Taunt</b>", set = "UNGORO", type = "minion", class = "druid", rarity = "rare", cost = 4, attack = 3, health = 5, tags = { "beast" }, keywords = { "taunt" } },
    { id = "UNG_101t", name = "Shellshifter", text = "<b>Stealth</b>", set = "UNGORO", type = "minion", class = "druid", rarity = "rare", cost = 4, attack = 5, health = 3, tags = { "beast" }, keywords = { "stealth" } },
    { id = "UNG_101t2", name = "Shellshifter", text = "<b>Taunt</b>", set = "UNGORO", type = "minion", class = "druid", rarity = "rare", cost = 4, attack = 3, health = 5, tags = { "beast" }, keywords = { "taunt" } },
    { id = "UNG_101t3", name = "Shellshifter", text = "<b>Stealth</b>\n<b>Taunt</b>", set = "UNGORO", type = "minion", class = "druid", rarity = "rare", cost = 4, attack = 5, health = 5, tags = { "beast" }, keywords = { "stealth", "taunt" } },
}
return card
